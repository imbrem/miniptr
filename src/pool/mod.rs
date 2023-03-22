/*!
A trait for simple allocators
*/

use std::marker::PhantomData;

use bytemuck::{TransparentWrapper, Zeroable};

use crate::index::ContiguousIx;

pub mod container;
pub mod slab;

/// A pool which supports inserting values of type `V` for keys of type `K`
pub trait Insert<K, V> {
    /// Insert `val` into the pool, assigning a new key which is returned
    ///
    /// Returns `val` as an error if the arena has run out of space
    #[must_use]
    fn try_insert(&mut self, val: V) -> Result<K, V>;

    /// Insert `v` into the pool, assigning a new key which is returned
    ///
    /// Panics if the pool has run out of space
    #[must_use]
    fn insert(&mut self, val: V) -> K {
        match self.try_insert(val) {
            Ok(key) => key,
            Err(_) => panic!("arena out of space"),
        }
    }
}

/// A pool indexed by keys of type `K` to values of type `V`
pub trait Pool<K> {
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

// A pool mapping keys of type `K` to values of type `V`
pub trait ObjectPool<K>: Pool<K> {
    /// The value type stored by this pool
    type Value;
}

/// A pool supporting insertion of objects, yielding keys
pub trait InsertPool<K>: ObjectPool<K> + Insert<K, Self::Value> {}
impl<K, P> InsertPool<K> for P where P: ObjectPool<K> + Insert<K, Self::Value> {}

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

/// A pool which supports removing a key and extracting values of type `V`
pub trait Take<K, V> {
    /// Try to take the value associated with key `k` from the mapping
    ///
    /// Returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[must_use]
    fn try_take(&mut self, key: K) -> Option<V>;

    /// Takes the value associated with key `k` from the mapping
    ///
    /// Panics or returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn take(&mut self, key: K) -> V {
        self.try_take(key).expect("cannot take unrecognized key")
    }
}

/// A [`Pool`] supporting the removal of keys
pub trait RemovePool<K>: ObjectPool<K> + Take<K, Self::Value> {
    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Guaranteed to have the same behaviour as [`Take<K, Self::Value`]'s `try_take` method,
    ///
    /// Returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_remove(&mut self, key: K) -> Option<Self::Value> {
        self.try_take(key)
    }

    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Guaranteed to have the same behaviour as [`Take<K, Self::Value`]'s `take` method,
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
        self.take(key)
    }
}

impl<P, K> RemovePool<K> for P where P: ObjectPool<K> + Take<K, Self::Value> {}

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
///
/// Automatically implemented for any [`Pool`] which implements [`GetRef<K, Self::Value>`].
///
/// Provides wrappers around [`GetRef`] methods returning `&Self::Value`, which might be useful for type inference.
pub trait PoolRef<K>: ObjectPool<K> + GetRef<K, Self::Value> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_get_value(&self, key: K) -> Option<&Self::Value> {
        self.try_get(key)
    }

    /// Get a reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_value(&self, key: K) -> &Self::Value {
        self.get(key)
    }
}
impl<P, K> PoolRef<K> for P where P: ObjectPool<K> + GetRef<K, Self::Value> {}

/// An [`Pool`] providing mutable access to values
///
/// Automatically implemented for any [`Pool`] which implements [`GetMut<K, Self::Value>`].
///
/// Provides wrappers around [`GetMut`] methods returning `&mut Self::Value`, which might be useful for type inference.
pub trait PoolMut<K>: ObjectPool<K> + GetMut<K, Self::Value> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_get_value_mut(&mut self, key: K) -> Option<&mut Self::Value> {
        self.try_get_mut(key)
    }

    /// Get a mutable reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_value_mut(&mut self, key: K) -> &mut Self::Value {
        self.get_mut(key)
    }
}
impl<P, K> PoolMut<K> for P where P: ObjectPool<K> + GetMut<K, Self::Value> {}

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
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, _key: K) {
        // This is a no-op, since all keys are "already deleted"
    }
}

impl<K, V> ObjectPool<K> for EmptyPool<V> {
    type Value = V;
}

impl<K, V> SafeFreePool<K> for EmptyPool<V> {}
impl<K, V> DoubleFreePool<K> for EmptyPool<V> {}

impl<K, V> Take<K, V> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_take(&mut self, _key: K) -> Option<V> {
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
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, _key: K) {}
}

impl<K, V> ObjectPool<K> for Arena<Vec<V>, K, ByClone>
where
    K: ContiguousIx,
{
    type Value = V;
}

impl<K, V> Pool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K) {
        if let Some(slot) = self.0.get_mut(key.index()) {
            *slot = Default::default()
        }
    }
}

impl<K, V> ObjectPool<K> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
    type Value = V;
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

impl<K, V> Take<K, V> for Arena<Vec<V>, K, ByDefault>
where
    K: ContiguousIx,
    V: Default,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_take(&mut self, key: K) -> Option<V>
    where
        V: Sized,
    {
        let r = self.0.get_mut(key.index())?;
        let mut result = V::default();
        std::mem::swap(&mut result, r);
        Some(result)
    }
}

impl<K, V> Take<K, V> for Arena<Vec<V>, K, ByClone>
where
    K: ContiguousIx,
    V: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_take(&mut self, key: K) -> Option<V>
    where
        V: Sized,
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

/// Forward implementations of [`Pool`], [`ObjectPool`], [`Insert`], [`Take`], [`GetRef`], and [`GetMut`] to a field of type `$P`
#[macro_export]
macro_rules! forward_pool_traits {
    (<$($gen:ident),*> $ty:ty => ($e:tt) : $P:ty) => {
        impl<$($gen,)* K> Pool<K> for $ty
        where
            $P: Pool<K>,
        {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn delete(&mut self, key: K) {
                self.$e.delete(key)
            }
        }

        impl<$($gen,)* K> ObjectPool<K> for $ty
        where
            $P: ObjectPool<K>,
        {
            type Value = P::Value;
        }

        impl<$($gen,)* K, V> Insert<K, V> for $ty
        where
            $P: Insert<K, V>,
        {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_insert(&mut self, val: V) -> Result<K, V> {
                self.$e.try_insert(val)
            }
        }

        impl<$($gen,)* K, V> GetRef<K, V> for $ty
        where
            $P: GetRef<K, V>,
        {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_get(&self, key: K) -> Option<&V> {
                self.$e.try_get(key)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn get(&self, key: K) -> &V {
                self.$e.get(key)
            }
        }

        impl<$($gen,)* K, V> GetMut<K, V> for $ty
        where
            $P: GetMut<K, V>,
        {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_get_mut(&mut self, key: K) -> Option<&mut V> {
                self.$e.try_get_mut(key)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn get_mut(&mut self, key: K) -> &mut V {
                self.$e.get_mut(key)
            }
        }

        impl<$($gen,)* K, V> Take<K, V> for $ty
        where
            $P: Take<K, V>,
        {
            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_take(&mut self, key: K) -> Option<V> {
                self.$e.try_take(key)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn take(&mut self, key: K) -> V {
                self.$e.take(key)
            }
        }
    };
    (<$($gen:ident),*> $ty:ty => $P:ty) => {
        $crate::forward_pool_traits!(<$($gen),*> $ty => (0): $P);
    };
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
        assert_eq!(arena.get(0), &0);
        *arena.get_mut(0) = 6;
        assert_eq!(arena.get(0), &6);
        assert_eq!(arena.try_get(1), None);
        assert_eq!(arena.try_remove(0), Some(6));
        assert_eq!(arena.try_remove(0), Some(0));
    }
}
