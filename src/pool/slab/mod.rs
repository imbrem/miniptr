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

use super::{
    container::{array::InsertFromSlice, Container, InsertEmpty, InsertWithCapacity, WithCapacity},
    GetMut, GetRef, Insert, ObjectPool, Pool, Take,
};

pub mod free;
use free::*;

/// A simple slab allocator supporting recycling of objects with a free-list
///
/// Allocates indices of type `K` corresponding to slots of type `S`
///
/// For a potentially more efficient free-list based approach, consider [`KeySlabPool`]
///
/// # Notes
///
/// The implementation of comparison will consider any two pools constructed by the same sequence of `insert` and `remove`/`delete` operations equivalent, but
/// may consider two pools which map the same keys to the same values but were constructed by a different sequence of operations to be disequal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlabPool<S, K = usize, F = KeyList<K>> {
    pool: Vec<S>,
    free_list: F,
    key_type: PhantomData<K>,
}

impl<S, K, F> Index<K> for SlabPool<S, K, F>
where
    S: SlotRef,
    K: ContiguousIx,
{
    type Output = S::Value;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn index(&self, index: K) -> &Self::Output {
        self.pool[index.index()].value()
    }
}

impl<S, K, F> IndexMut<K> for SlabPool<S, K, F>
where
    S: SlotMut + SlotRef,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.pool[index.index()].value_mut()
    }
}

impl<S, K, F> SlabPool<S, K, F>
where
    S: Slot,
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    /// Create a new, empty pool
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn new() -> SlabPool<S, K, F>
    where
        F: Default,
    {
        SlabPool {
            pool: Vec::new(),
            free_list: F::default(),
            key_type: PhantomData,
        }
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

    // /// Create a new, empty pool with the given capacity
    // #[cfg_attr(not(tarpaulin), inline)]
    // pub fn with_capacity(capacity: usize, free_capacity: usize) -> SlabPool<S, K> {
    //     SlabPool {
    //         pool: Vec::with_capacity(capacity),
    //         free_list: Vec::with_capacity(free_capacity),
    //     }
    // }

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

    /// Get the number of free slots in this pool.
    ///
    /// Note this is less than or equal to the free capacity
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn free_slots(&self) -> usize
    where
        F: FreeListCapacity<[S], K>,
    {
        self.free_list.len(&self.pool)
    }

    /// Get the free capacity of this pool. May take time linear in the size of the pool.
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn free_capacity(&self) -> usize
    where
        F: FreeListCapacity<[S], K>,
    {
        self.free_slots() + self.capacity() - self.total_slots()
    }

    /// Remove all entries from this pool, preserving its current capacity
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn clear(&mut self) {
        self.free_list.clear(&mut self.pool);
        self.pool.clear();
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn reserve(&mut self, additional: usize) {
        self.pool.reserve(additional)
    }

    // /// Reserves capacity for at least `additional` more elements to be free'd
    // #[cfg_attr(not(tarpaulin), inline)]
    // pub fn reserve_free(&mut self, additional: usize) {
    //     self.free_list.reserve(additional)
    // }

    /// Shrink this pool's capacity as much as possible without changing any indices
    #[cfg_attr(not(tarpaulin), inline)]
    pub fn shrink_to_fit(&mut self) {
        self.pool.shrink_to_fit();
        // self.free_list.shrink_to_fit();
    }

    /// Get the key that will be assigned to the next inserted value, or `None` if inserting a new value would cause the pool to overflow
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn next_key(&self) -> Option<K>
    where
        F: NextFreeList<[S], K>,
    {
        if let Some(next_free) = self.free_list.next_free(&self.pool) {
            return Some(next_free);
        }
        K::try_new(self.pool.len())
    }
}

impl<S, K, V, F> Insert<K, V> for SlabPool<S, K, F>
where
    S: Slot + InitFrom<V>,
    K: ContiguousIx,
    F: FreeList<[S], K>,
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
        if let Some(free) = self.free_list.alloc(&mut self.pool) {
            self.pool[free.index()].set_value(v);
            Ok(free)
        } else if let Some(ix) = K::try_new(self.pool.len()) {
            self.pool.push(S::from_value(v));
            Ok(ix)
        } else {
            Err(v)
        }
    }
}

impl<S, K, F> InsertEmpty<K> for SlabPool<S, K, F>
where
    S: Slot,
    S::Value: Container + Default,
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    #[inline]
    fn try_insert_empty(&mut self) -> Result<K, ()> {
        if let Some(free) = self.free_list.alloc(&mut self.pool) {
            self.pool[free.index()].set_default_value();
            Ok(free)
        } else if let Some(ix) = K::try_new(self.pool.len()) {
            self.pool.push(S::default_value());
            Ok(ix)
        } else {
            Err(())
        }
    }

    #[inline]
    fn insert_unique_empty(&mut self) -> Result<K, ()> {
        if let Some(free) = self.free_list.alloc(&mut self.pool) {
            self.pool[free.index()].set_default_value();
            Ok(free)
        } else if let Some(ix) = K::try_new(self.pool.len()) {
            self.pool.push(S::default_value());
            Ok(ix)
        } else {
            Err(())
        }
    }
}

impl<S, K, C, F> InsertWithCapacity<K, C> for SlabPool<S, K, F>
where
    S: Slot,
    S::Value: Container + WithCapacity<C>,
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn insert_with_capacity(&mut self, capacity: C) -> K {
        self.insert(WithCapacity::new_with_capacity(capacity))
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_insert_with_capacity(&mut self, capacity: C) -> Result<K, ()> {
        self.try_insert(WithCapacity::new_with_capacity(capacity))
            .map_err(|_| ())
    }
}

impl<'a, S, K, F> InsertFromSlice<'a, K> for SlabPool<S, K, F>
where
    S: Slot,
    S::Value: Container + From<&'a [Self::Elem]>,
    Self::Elem: 'a,
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn insert_from_slice(&mut self, slice: &'a [Self::Elem]) -> K {
        self.insert(S::Value::from(slice))
    }
}

impl<S, K, F> Pool<K> for SlabPool<S, K, F>
where
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K) {
        self.free_list.delete(key, &mut self.pool);
    }
}

impl<S, K, F> ObjectPool<K> for SlabPool<S, K, F>
where
    S: Slot,
    K: ContiguousIx,
    F: FreeList<[S], K>,
{
    type Object = S::Value;
}

impl<S, K, F> Take<K, S::Value> for SlabPool<S, K, F>
where
    S: Slot,
    K: ContiguousIx,
    F: RemovalList<[S], K, Value = S::Value>,
{
    #[inline]
    fn try_take(&mut self, key: K) -> Option<S::Value> {
        self.free_list.try_remove(key, &mut self.pool)
    }

    #[inline]
    fn take(&mut self, key: K) -> S::Value {
        self.free_list.remove(key, &mut self.pool)
    }
}

impl<S, K, F> GetRef<K, S::Value> for SlabPool<S, K, F>
where
    S: SlotRef,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_at(&self, key: K) -> Option<&S::Value> {
        self.pool.get(key.index())?.try_value()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn at(&self, key: K) -> &S::Value {
        self.pool[key.index()].value()
    }
}

impl<S, K, F> GetMut<K, S::Value> for SlabPool<S, K, F>
where
    S: SlotMut,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_at_mut(&mut self, key: K) -> Option<&mut S::Value> {
        self.pool.get_mut(key.index())?.try_value_mut()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn at_mut(&mut self, key: K) -> &mut S::Value {
        self.pool[key.index()].value_mut()
    }
}

pub type KeySlabPool<S, K = usize> = SlabPool<S, K, IntrusiveFree>;

#[cfg(test)]
mod test {
    use crate::pool::container::map::{GetIndex, GetIndexMut};
    use crate::pool::container::stack::StackPool;
    use crate::pool::container::{IsEmptyPool, LenPool};
    use crate::pool::RemovePool;
    use crate::slot::{CloneSlot, DefaultSlot};

    use super::*;
    use either::Either;
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoshiro256StarStar;

    #[test]
    fn basic_slab_pool_usage() {
        let mut pool: SlabPool<DefaultSlot<String>, u8> = SlabPool::new();
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
            assert_eq!(pool.get_slot(i).unwrap().0, s);
            assert_eq!(pool.get_slot_mut(i).unwrap().0, s);
            assert_eq!(pool.at(i), &s);
            assert_eq!(pool.at_mut(i), &mut s);
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
            if i % 2 != 0 {
                let mut s = format!("{i}a");
                assert_eq!(v.0, s);
                assert_eq!(pool.at(i), &s);
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
            assert_eq!(pool.get_slot(keys[i as usize]).unwrap().0, s);
            assert_eq!(pool.get_slot_mut(keys[i as usize]).unwrap().0, s);
            assert_eq!(pool.at(keys[i as usize]), &s);
            assert_eq!(pool.try_at(keys[i as usize]), Some(&s));
            assert_eq!(pool.at_mut(keys[i as usize]), &s);
            assert_eq!(pool.try_at_mut(keys[i as usize]), Some(&mut s));
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
    fn slab_insertion_overflow() {
        let mut pool: SlabPool<DefaultSlot<usize>, u8> = SlabPool::new();
        for i in 0..257 {
            let _ = pool.insert(i);
        }
    }

    #[test]
    fn basic_key_slab_pool_usage() {
        let mut pool: KeySlabPool<Either<u8, String>, u8> = KeySlabPool::new();
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
            assert_eq!(pool.at(i), &s);
            assert_eq!(pool.at_mut(i), &mut s);
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
                assert_eq!(pool.at(i), &s);
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
            assert_eq!(pool.at(keys[i as usize]), &s);
            assert_eq!(pool.try_at(keys[i as usize]), Some(&s));
            assert_eq!(pool.at_mut(keys[i as usize]), &s);
            assert_eq!(pool.try_at_mut(keys[i as usize]), Some(&mut s));
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
    fn key_slab_insertion_overflow() {
        let mut pool: SlabPool<DefaultSlot<usize>, u8> = SlabPool::new();
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
        let mut trace_pool: KeySlabPool<Either<usize, isize>> = KeySlabPool::new();
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

        let mut pool: KeySlabPool<Either<usize, usize>> = KeySlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(event as usize, pool.insert(event as usize));
            } else {
                assert_eq!(-(event + 1) as usize, pool.remove(-(event + 1) as usize));
            }
        }

        let mut pool: KeySlabPool<DefaultSlot<usize>> = KeySlabPool::new();
        for &event in trace.iter() {
            if event >= 0 {
                assert_eq!(event as usize, pool.insert(event as usize));
            } else {
                assert_eq!(-(event + 1) as usize, pool.remove(-(event + 1) as usize));
            }
        }

        let mut pool: KeySlabPool<Either<usize, usize>> = KeySlabPool::new();
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

        let mut pool: KeySlabPool<DefaultSlot<usize>> = KeySlabPool::new();
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

        let mut pool: SlabPool<DefaultSlot<usize>> = SlabPool::new();
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

        let mut pool: SlabPool<CloneSlot<usize>> = SlabPool::new();
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

    #[test]
    fn slab_stack_pool() {
        let mut pool: SlabPool<DefaultSlot<Vec<u32>>> = SlabPool::new();
        let s1 = pool.insert_empty();
        assert_eq!(s1, 0);
        assert!(pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 0);
        pool.push(s1, 5);
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 1);
        pool.push(s1, 6);
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 2);
        assert_eq!(pool.pop(s1), Some(6));
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 1);
        let s2 = pool.insert_unique_empty().unwrap();
        assert_eq!(s2, 1);
        let s3 = pool.insert_with_capacity(3);
        assert_eq!(s3, 2);
        let s4 = pool.insert_with_capacity(4);
        assert_eq!(s4, 3);
        assert_eq!(pool.remove(s4), vec![]);
        let s4 = pool.insert_empty();
        assert_eq!(s4, 3);
        assert_eq!(s4, pool.into_pushed(s4, 5));
        assert_eq!(Some((s4, 5)), pool.into_popped(s4));
        assert_eq!(None, pool.into_popped(s4));

        let mut small_pool: SlabPool<DefaultSlot<Vec<u32>>, u8> = SlabPool::new();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.try_insert_empty());
        }
        assert_eq!(Err(()), small_pool.try_insert_empty());
        small_pool.clear();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.insert_unique_empty());
        }
        assert_eq!(Err(()), small_pool.insert_unique_empty());
        small_pool.clear();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.try_insert_with_capacity(i as usize));
        }
        assert_eq!(Err(()), small_pool.try_insert_with_capacity(3));
    }

    #[test]
    fn key_slab_stack_pool() {
        let mut pool: KeySlabPool<Either<usize, Vec<u32>>> = KeySlabPool::new();
        let s1 = pool.insert_empty();
        assert_eq!(s1, 0);
        assert!(pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 0);
        pool.push(s1, 5);
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 1);
        pool.push(s1, 6);
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 2);
        assert_eq!(pool.pop(s1), Some(6));
        assert!(!pool.key_is_empty(s1));
        assert_eq!(pool.key_len(s1), 1);
        assert_eq!(pool.get_index(s1, 0), Some(&5));
        assert_eq!(pool.get_index_unchecked(s1, 0), &5);
        assert_eq!(pool.get_index_mut(s1, 0), Some(&mut 5));
        assert_eq!(pool.get_index_mut_unchecked(s1, 0), &mut 5);
        assert_eq!(pool.get_index(s1, 1), None);
        let s2 = pool.insert_unique_empty().unwrap();
        assert_eq!(s2, 1);
        let s3 = pool.insert_with_capacity(3);
        assert_eq!(s3, 2);
        let s4 = pool.insert_with_capacity(4);
        assert_eq!(s4, 3);
        assert_eq!(pool.remove(s4), vec![]);
        let s4 = pool.insert_empty();
        assert_eq!(s4, 3);
        assert_eq!(s4, pool.into_pushed(s4, 5));
        assert_eq!(Some((s4, 5)), pool.into_popped(s4));
        assert_eq!(None, pool.into_popped(s4));

        assert_eq!(pool.key_capacity(s4), pool.at(s4).capacity());

        let mut small_pool: SlabPool<DefaultSlot<Vec<u32>>, u8> = SlabPool::new();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.try_insert_empty());
        }
        assert_eq!(Err(()), small_pool.try_insert_empty());
        small_pool.clear();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.insert_unique_empty());
        }
        assert_eq!(Err(()), small_pool.insert_unique_empty());
        small_pool.clear();
        for i in 0..=255 {
            assert_eq!(Ok(i), small_pool.try_insert_with_capacity(i as usize));
        }
        assert_eq!(Err(()), small_pool.try_insert_with_capacity(3));
    }
}
