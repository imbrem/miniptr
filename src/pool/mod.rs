/*!
A trait for simple allocators
*/

use std::marker::PhantomData;

use bytemuck::{TransparentWrapper, Zeroable};

use crate::index::ContiguousIx;

pub mod list;
pub mod slab;

/// A pool which supports inserting values of type `V` for keys of type `K`
pub trait Insert<K, V> {
    /// Insert `val` into the pool, assigning a new key which is returned
    ///
    /// Returns `val` as an error if the arena has run out of space
    #[must_use]
    fn try_insert(&mut self, val: V) -> Result<K, V>
    where
        V: Sized;

    /// Insert `v` into the pool, assigning a new key which is returned
    ///
    /// Panics if the pool has run out of space
    #[must_use]
    fn insert(&mut self, val: V) -> K
    where
        V: Sized,
    {
        match self.try_insert(val) {
            Ok(key) => key,
            Err(_) => panic!("arena out of space"),
        }
    }
}

/// A pool mapping keys of type `K` to values of type `V`
pub trait Pool<K>: Insert<K, Self::Value> {
    /// The value type stored by this pool
    type Value;

    /// Deletes the key `k` from the mapping.
    ///
    /// Note that this is *not* guaranteed to do anything; in pools which do not support the removal of keys, this may simply be a no-op.
    ///
    /// Depending on the implementation of `Pool`, this may be slightly more efficient than `remove` since resources used may be more effectively recycled.
    ///
    /// Leaves `self` in an unspecified but valid state or panics if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    ///
    /// This method is provided as a convenience for users who do not need to retrieve the value associated with the key and simply want to remove it from the pool.
    fn delete(&mut self, key: K);
}

/// A [`Pool`] for which `Pool::delete` may be called multiple times on the same key without modifying any other key; panicking is allowed.
///
/// If [`RemovePool`] is also implemented, `Self::remove` may still return an arbitrary value on a deleted key, but may *not* modify any other key.
pub trait SafeFreePool<K>: Pool<K> {}

/// A [`Pool`] for which `Pool::delete` may be called multiple times on the same key without panicking or modifying any other key.
///
/// If [`RemovePool`] is also implemented, `Self::remove` may still return an arbitrary value on a deleted key, but may *not* panic or modify any other key.
/// For stronger bounds on this, consider requiring [`DoubleRemovePool`].
pub trait DoubleFreePool<K>: SafeFreePool<K> {}

/// A [`Pool`] for which `Pool::remove` may be called multiple times on the same key, and is guaranteed to return `None` on previously removed keys
pub trait DoubleRemovePool<K>: DoubleFreePool<K> {}

/// A [`Pool`] supporting the removal of keys
pub trait RemovePool<K>: Pool<K> {
    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[must_use]
    fn try_remove(&mut self, key: K) -> Option<Self::Value>
    where
        Self::Value: Sized;

    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Panics or returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn remove(&mut self, key: K) -> Self::Value
    where
        Self::Value: Sized,
    {
        self.try_remove(key)
            .expect("cannot remove unrecognized key")
    }
}

/// A pool providing read-only access to values of type `V` given a key of type `K`
pub trait GetRef<K, V> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[must_use]
    fn try_get(&self, key: K) -> Option<&V>;

    /// Get a reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get(&self, key: K) -> &V {
        self.try_get(key).expect("cannot get unrecognized key")
    }
}

/// An pool providing mutable access to values given a key of type `K`
pub trait GetMut<K, V> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[must_use]
    fn try_get_mut(&mut self, key: K) -> Option<&mut V>;

    /// Get a mutable reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_mut(&mut self, key: K) -> &mut V {
        self.try_get_mut(key)
            .expect("cannot mutably get unrecognized key")
    }
}

/// A [`Pool`] providing read-only access to values
pub trait PoolRef<K>: Pool<K> + GetRef<K, Self::Value> {}
impl<P, K> PoolRef<K> for P where P: Pool<K> + GetRef<K, Self::Value> {}

/// An [`Pool`] providing mutable access to values
pub trait PoolMut<K>: Pool<K> + GetMut<K, Self::Value> {}
impl<P, K> PoolMut<K> for P where P: Pool<K> + GetMut<K, Self::Value> {}

/// A [`Pool`] which does not contain any values, and is always full
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Zeroable)]
pub struct EmptyPool<V>(PhantomData<V>);

impl<K, V> Insert<K, V> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_insert(&mut self, val: V) -> Result<K, V> {
        Err(val)
    }
}

impl<K, V> Pool<K> for EmptyPool<V> {
    type Value = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, _key: K) {
        // This is a no-op, since all keys are "already deleted"
    }
}

impl<K, V> SafeFreePool<K> for EmptyPool<V> {}
impl<K, V> DoubleFreePool<K> for EmptyPool<V> {}

impl<K, V> RemovePool<K> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove(&mut self, _key: K) -> Option<V> {
        None
    }
}

impl<K, V> DoubleRemovePool<K> for EmptyPool<V> {}

impl<K, V> GetRef<K, V> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get(&self, _key: K) -> Option<&V> {
        None
    }
}

impl<K, V> GetMut<K, V> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get_mut(&mut self, _key: K) -> Option<&mut V> {
        None
    }
}

/// A wrapper around [`Vec`] implementing an arena allocator for a type `V`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, TransparentWrapper)]
#[repr(transparent)]
#[transparent(V)]
pub struct Arena<V, K = usize, D = ByDefault>(V, PhantomData<K>, PhantomData<D>);

/// Remove a value from this arena by cloning it out
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Zeroable)]
pub struct ByClone;

/// Remove a value from this arena by replacing it with `Default`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Zeroable)]
pub struct ByDefault;

impl<V> Arena<Vec<V>> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn new(value: Vec<V>) -> Arena<Vec<V>> {
        Arena(value, PhantomData, PhantomData)
    }
}

impl<V, K, D> From<V> for Arena<V, K, D> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from(value: V) -> Self {
        Arena(value, PhantomData, PhantomData)
    }
}

impl<K, V, D> Insert<K, V> for Arena<Vec<V>, K, D>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_insert(&mut self, val: V) -> Result<K, V>
    where
        V: Sized,
    {
        let Some(ix) = K::try_new(self.0.len()) else { return Err(val) };
        self.0.push(val);
        Ok(ix)
    }
}

impl<K, V> Pool<K> for Arena<Vec<V>, K, ByClone>
where
    K: ContiguousIx,
{
    type Value = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, _key: K) {}
}

impl<K, V> Pool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
    type Value = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K) {
        if let Some(slot) = self.0.get_mut(key.index()) {
            *slot = Default::default()
        }
    }
}

impl<K, V> SafeFreePool<K> for Arena<Vec<V>, K, ByClone> where K: ContiguousIx {}
impl<K, V> DoubleFreePool<K> for Arena<Vec<V>, K, ByClone> where K: ContiguousIx {}
impl<K, V> SafeFreePool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
}
impl<K, V> DoubleFreePool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
}

impl<K, V> RemovePool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove(&mut self, key: K) -> Option<Self::Value>
    where
        Self::Value: Sized,
    {
        let r = self.0.get_mut(key.index())?;
        let mut result = V::default();
        std::mem::swap(&mut result, r);
        Some(result)
    }
}

impl<K, V> RemovePool<K> for Arena<Vec<V>, K, ByClone>
where
    K: ContiguousIx,
    V: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove(&mut self, key: K) -> Option<Self::Value>
    where
        Self::Value: Sized,
    {
        self.0.get(key.index()).cloned()
    }
}

impl<K, V> GetRef<K, V> for Arena<Vec<V>, K>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get(&self, key: K) -> Option<&V> {
        self.0.get(key.index())
    }
}

impl<K, V> GetMut<K, V> for Arena<Vec<V>, K>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get_mut(&mut self, key: K) -> Option<&mut V> {
        self.0.get_mut(key.index())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_pool_insert_fails() {
        let _: u8 = EmptyPool::<u64>::default().insert(5);
    }

    #[test]
    #[should_panic]
    fn empty_pool_remove_fails() {
        let _ = EmptyPool::<u64>::default().remove(5);
    }

    #[test]
    fn empty_pool_delete() {
        let _ = EmptyPool::<u64>::default().delete(5);
    }

    #[test]
    #[should_panic]
    fn empty_pool_get_fails() {
        let _ = EmptyPool::<u64>::default().get(5);
    }

    #[test]
    #[should_panic]
    fn empty_pool_get_mut_fails() {
        let _ = EmptyPool::<u64>::default().get_mut(5);
    }

    #[test]
    fn basic_arena_usage() {
        let mut arena = Arena::new(vec![]);
        assert_eq!(arena.insert(5), 0);
        arena.delete(0);
        assert_eq!(arena.get(0), &5);
        *arena.get_mut(0) = 6;
        assert_eq!(arena.get(0), &6);
        assert_eq!(arena.try_get(1), None);
        assert_eq!(arena.try_remove(0), Some(6));
        assert_eq!(arena.try_remove(0), Some(0));
    }
}
