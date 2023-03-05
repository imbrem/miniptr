/*!
A slab allocator, returning pointers to pre-allocated storage of a uniformly sized type
*/
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    index::ContiguousIx,
    slot::{InitFrom, Slot, SlotMut, SlotRef},
};

use super::{Pool, Insert, PoolMut, PoolRef};

/// A simple slab allocator supporting recycling of objects with a free-list
///
/// Allocates indices of type `K` corresponding to slots of type `S`
///
/// # Notes
///
/// The implementation of comparison will consider any two pools constructed by the same sequence of `insert` and `remove`/`delete` operations equivalent, but
/// may consider two pools which map the same keys to the same values but were constructed by a different sequence of operations to be disequal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlabPool<S, K = usize> {
    pool: Vec<S>,
    free_head: usize,
    phantom_free: PhantomData<K>,
}

impl<S, K> Index<K> for SlabPool<S, K>
where
    S: SlotRef<K>,
    K: ContiguousIx,
{
    type Output = S::Value;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn index(&self, index: K) -> &Self::Output {
        self.pool[index.index()].value()
    }
}

impl<S, K> IndexMut<K> for SlabPool<S, K>
where
    S: SlotMut<K> + SlotRef<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.pool[index.index()].value_mut()
    }
}

impl<S, K> SlabPool<S, K>
where
    S: Slot<K>,
    K: ContiguousIx,
{
    /// Create a new, empty pool
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn new() -> SlabPool<S, K> {
        Self::with_capacity(0)
    }

    /// Get a reference to a given slot
    ///
    /// Note this may expose unstable internal details of the pool data structure when used on a key which has been deleted.
    /// Using interior mutability to modify the slot corresponding to a deleted key leaves the pool in an invalid state, though this will never cause UB.
    ///
    /// Returns `None` if `key` is invalid
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn get_slot(&self, key: K) -> Option<&S> {
        self.pool.get(key.index())
    }

    /// Get a mutable reference to a given slot
    ///
    /// Note this may expose unstable internal details of the pool data structure when used on a key which has been deleted.
    /// Modifying the slot corresponding to a deleted key leaves the pool in an invalid state, though this will never cause UB.
    ///
    /// Returns `None` if `key` is invalid
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn get_slot_mut(&mut self, key: K) -> Option<&mut S> {
        self.pool.get_mut(key.index())
    }

    /// Create a new, empty pool with the given capacity
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn with_capacity(capacity: usize) -> SlabPool<S, K> {
        SlabPool {
            pool: Vec::with_capacity(capacity),
            free_head: 0,
            phantom_free: PhantomData,
        }
    }

    /// Get the total capacity of this pool
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }

    /// Get the total number of slots in this pool
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn total_slots(&self) -> usize {
        self.pool.len()
    }

    /// Get the number of free slots in this pool. May take time linear in the size of the pool.
    ///
    /// Note this is less than or equal to the free capacity
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn free_slots(&self) -> usize {
        let mut curr = self.free_head;
        let mut count = 0;
        while let Some(ix) = self.pool.get(curr) {
            count += 1;
            let next = ix.key().index();
            if next == curr {
                break;
            }
            curr = next;
        }
        count
    }

    /// Get the free capacity of this pool. May take time linear in the size of the pool.
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn free_capacity(&self) -> usize {
        self.free_slots() + self.capacity() - self.total_slots()
    }

    /// Remove all entries from this pool, preserving its current capacity
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn clear(&mut self) {
        self.pool.clear();
        self.free_head = 0;
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn reserve(&mut self, additional: usize) {
        self.pool.reserve(additional)
    }

    /// Shrink this pool's capacity as much as possible without changing any indices
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn shrink_to_fit(&mut self) {
        self.pool.shrink_to_fit();
    }

    /// Get the key that will be assigned to the next inserted value, or `None` if inserting a new value would cause the pool to overflow
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn next_key(&self) -> Option<K> {
        K::try_new(self.free_head)
    }
}

impl<S, K, V> Insert<K, V> for SlabPool<S, K>
where
    S: Slot<K> + InitFrom<V>,
    K: ContiguousIx,
{
    #[inline]
    fn insert(&mut self, v: V) -> K {
        match self.try_insert(v) {
            Ok(k) => k,
            Err(_) => panic!(
                "Slab mapping out of space: current size {:?}",
                self.pool.len()
            ),
        }
    }

    #[inline]
    fn try_insert(&mut self, v: V) -> Result<K, V> {
        let Some(key) = self.next_key() else { return Err(v) };
        self.free_head = if let Some(value) = self.pool.get_mut(self.free_head) {
            let key = value.key();
            value.set_value(v);
            let ki = key.index();
            if self.free_head == ki {
                self.pool.len()
            } else {
                ki
            }
        } else {
            self.pool.push(S::from_value(v));
            self.pool.len()
        };
        Ok(key)
    }
}

impl<S, K> Pool<K> for SlabPool<S, K>
where
    S: Slot<K>,
    K: ContiguousIx,
{
    type Value = S::Value;

    #[inline]
    fn try_remove(&mut self, key: K) -> Option<Self::Value> {
        let f = if self.free_head < self.pool.len() {
            K::new(self.free_head)
        } else {
            key
        };
        let ki = key.index();
        let result = self.pool.get_mut(ki)?.try_swap_key(f)?;
        self.free_head = ki;
        Some(result)
    }

    #[inline]
    fn remove(&mut self, key: K) -> Self::Value {
        let f = if self.free_head < self.pool.len() {
            K::new(self.free_head)
        } else {
            key
        };
        let ki = key.index();
        let result = self.pool[ki].swap_key(f);
        self.free_head = ki;
        result
    }

    #[inline]
    fn delete(&mut self, key: K) {
        let f = if self.free_head < self.pool.len() {
            K::new(self.free_head)
        } else {
            key
        };
        let ki = key.index();
        self.pool[ki].set_key(f);
        self.free_head = ki;
    }
}

impl<S, K> PoolRef<K> for SlabPool<S, K>
where
    S: SlotRef<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get(&self, key: K) -> Option<&Self::Value> {
        self.pool.get(key.index())?.try_value()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get(&self, key: K) -> &Self::Value {
        self.pool[key.index()].value()
    }
}

impl<S, K> PoolMut<K> for SlabPool<S, K>
where
    S: SlotMut<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get_mut(&mut self, key: K) -> Option<&mut Self::Value> {
        self.pool.get_mut(key.index())?.try_value_mut()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_mut(&mut self, key: K) -> &mut Self::Value {
        self.pool[key.index()].value_mut()
    }
}

#[cfg(test)]
mod test {
    use crate::slot::IdSlot;

    use super::*;
    use either::Either;
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoshiro256StarStar;

    #[test]
    fn basic_pool_usage() {
        let mut pool: SlabPool<Either<u8, String>, u8> = SlabPool::new();
        assert_eq!(pool.total_slots(), 0);
        assert_eq!(pool.free_slots(), 0);
        assert_eq!(pool.capacity(), 0);
        assert_eq!(pool.free_capacity(), 0);

        pool.reserve(3);

        assert_eq!(pool.total_slots(), 0);
        assert_eq!(pool.free_slots(), 0);
        assert!(pool.capacity() >= 3);
        assert_eq!(pool.free_capacity(), pool.capacity());

        // Test insertion
        for i in 0..=255u8 {
            assert_eq!(pool.next_key(), Some(i));
            let k = pool.insert(format!("{i}"));
            assert_eq!(k, i);
        }
        assert_eq!(pool.next_key(), None);
        assert_eq!(pool.try_insert("256".to_string()), Err("256".to_string()));

        assert_eq!(pool.total_slots(), 256);
        assert_eq!(pool.free_slots(), 0);
        let cap = pool.capacity();
        assert!(cap >= 256);
        assert_eq!(pool.free_capacity(), cap - pool.total_slots());

        // Test reading
        for i in 0..=255u8 {
            let mut s = format!("{i}");
            assert_eq!(pool.get_slot(i).unwrap().as_ref(), Either::Right(&s));
            assert_eq!(pool.get_slot_mut(i).unwrap().as_ref(), Either::Right(&s));
            assert_eq!(pool.get(i), &s);
            assert_eq!(pool.get_mut(i), &mut s);
            assert_eq!(pool[i], s);
            s.push('a');
            pool[i].push('a');
            assert_eq!(&mut pool[i], &mut s);
        }

        // Test deleting
        let mut free = 0;
        for i in (0..=255u8).step_by(2) {
            assert_eq!(pool.free_slots(), free);
            assert_eq!(pool.remove(i), format!("{i}a"));
            free += 1;
            assert_eq!(pool.free_slots(), free);
        }
        assert_eq!(pool.total_slots(), 256);
        assert_eq!(pool.free_slots(), 128);
        assert_eq!(pool.capacity(), cap);

        for i in 0..=255u8 {
            let v = pool.get_slot(i).unwrap();
            if i == 0 {
                assert_eq!(v, &Either::Left(0));
            } else if i % 2 == 0 {
                assert_eq!(v, &Either::Left(i - 2));
            } else {
                let mut s = format!("{i}a");
                assert_eq!(v.as_ref(), Either::Right(&s));
                assert_eq!(pool.get(i), &s);
                assert_eq!(pool[i], s);
                s.push('b');
                pool[i].push('b');
                assert_eq!(&mut pool[i], &mut s);
            }
        }

        // Test deleting everything
        for i in (1..=255u8).step_by(2) {
            pool.delete(i);
        }

        assert_eq!(pool.total_slots(), 256);
        assert_eq!(pool.free_slots(), 256);
        assert_eq!(pool.capacity(), cap);
        assert_eq!(pool.free_capacity(), cap);

        // Test re-inserting everything
        let mut keys = Vec::new();

        for i in 0..=255u8 {
            assert!(pool.next_key().is_some());
            keys.push(pool.insert(format!("{i}c")));
        }
        assert_eq!(pool.next_key(), None);
        assert_eq!(pool.try_insert("256".to_string()), Err("256".to_string()));

        assert_eq!(pool.total_slots(), 256);
        assert_eq!(pool.free_slots(), 0);
        assert_eq!(pool.capacity(), cap);

        // Test reading
        for i in 0..=255u8 {
            let mut s = format!("{i}c");
            assert_eq!(
                pool.get_slot(keys[i as usize]).unwrap().as_ref(),
                Either::Right(&s)
            );
            assert_eq!(
                pool.get_slot_mut(keys[i as usize]).unwrap().as_ref(),
                Either::Right(&s)
            );
            assert_eq!(pool.get(keys[i as usize]), &s);
            assert_eq!(pool.try_get(keys[i as usize]), Some(&s));
            assert_eq!(pool.get_mut(keys[i as usize]), &s);
            assert_eq!(pool.try_get_mut(keys[i as usize]), Some(&mut s));
            assert_eq!(pool[keys[i as usize]], s);
            s.push('d');
            pool[keys[i as usize]].push('d');
            assert_eq!(&mut pool[keys[i as usize]], &mut s);
        }

        // Test shrinking and clearing
        pool.shrink_to_fit();
        assert_eq!(pool.total_slots(), 256);
        assert_eq!(pool.free_slots(), 0);
        assert_eq!(pool.capacity(), cap);
        pool.clear();
        assert_eq!(pool.total_slots(), 0);
        assert_eq!(pool.free_slots(), 0);
        assert_eq!(pool.capacity(), cap);
        assert_eq!(pool.free_capacity(), cap);
        pool.shrink_to_fit();
        assert_eq!(pool.total_slots(), 0);
        assert_eq!(pool.free_slots(), 0);
        assert_eq!(pool.capacity(), 0);
        assert_eq!(pool.free_capacity(), 0);
    }

    #[test]
    #[should_panic]
    fn insertion_overflow() {
        let mut pool: SlabPool<Either<u8, usize>, u8> = SlabPool::new();
        for i in 0..257 {
            let _ = pool.insert(i);
        }
    }

    #[test]
    fn insertion_removal_stress() {
        const REMOVAL_FRACTION: f64 = 0.3;
        const SIZE: usize = 1000;
        let mut rng = Xoshiro256StarStar::from_seed([0xAB; 32]);
        let mut trace: Vec<isize> = Vec::new();
        let mut trace_pool: SlabPool<Either<usize, isize>> = SlabPool::new();
        let mut inserted = Vec::new();
        for _ in 0..SIZE {
            if !inserted.is_empty() && rng.gen_bool(REMOVAL_FRACTION) {
                let remove = inserted.swap_remove(rng.gen_range(0..inserted.len()));
                let _ = trace_pool.remove(remove);
                trace.push(-(remove as isize) - 1)
            } else {
                let key = trace_pool.insert(0);
                inserted.push(key);
                trace.push(key as isize)
            }
        }

        let mut pool: SlabPool<Either<usize, usize>> = SlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(event as usize, pool.insert(event as usize));
            } else {
                assert_eq!(-(event + 1) as usize, pool.remove(-(event + 1) as usize));
            }
        }

        let mut pool: SlabPool<IdSlot<usize>> = SlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(event as usize, pool.insert(event as usize));
            } else {
                assert_eq!(-(event + 1) as usize, pool.remove(-(event + 1) as usize));
            }
        }

        let mut pool: SlabPool<Either<usize, usize>> = SlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(Ok(event as usize), pool.try_insert(event as usize));
            } else {
                assert_eq!(
                    Some(-(event + 1) as usize),
                    pool.try_remove(-(event + 1) as usize)
                );
            }
        }

        let mut pool: SlabPool<IdSlot<usize>> = SlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(Ok(event as usize), pool.try_insert(event as usize));
            } else {
                assert_eq!(
                    Some(-(event + 1) as usize),
                    pool.try_remove(-(event + 1) as usize)
                );
            }
        }
    }
}
