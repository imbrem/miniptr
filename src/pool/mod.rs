/*!
A trait for arena-like allocators
*/

use std::marker::PhantomData;

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

/// A pool mapping keys of type `K` to values of type `V`
pub trait Pool<K>: Insert<K, Self::Value> {
    /// The value stored by this allocator
    type Value;

    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[must_use]
    fn try_remove(&mut self, key: K) -> Option<Self::Value>;

    /// Deletes the key `k` from the mapping, returning its value.
    ///
    /// Panics or returns an arbitrary value, leaving `self` in an unspecified but valid state, if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn remove(&mut self, key: K) -> Self::Value {
        self.try_remove(key)
            .expect("cannot remove unrecognized key")
    }

    /// Deletes the key `k` from the mapping.
    ///
    /// Depending on the implementation of `Pool`, this may be slightly more efficient than `remove` since resources used may be more effectively recycled.
    ///
    /// Leaves `self` in an unspecified but valid state or panics if `k` is unrecognized.
    /// `k` is considered unrecognized if it:
    /// - Was not returned from `self.insert` or `self.try_insert`
    /// - Has already been deleted
    ///
    /// This method is provided as a convenience for users who do not need to retrieve the value associated with the key and simply want to remove it from the pool.
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K) {
        let _ = self.remove(key);
    }
}

/// A [`Pool`] providing read-only access to values
pub trait PoolRef<K>: Pool<K> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[must_use]
    fn try_get(&self, key: K) -> Option<&Self::Value>;

    /// Get a reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get(&self, key: K) -> &Self::Value {
        self.try_get(key).expect("cannot get unrecognized key")
    }
}

/// An [`Pool`] providing mutable access to values
pub trait PoolMut<K>: Pool<K> {
    /// Try to get a reference to the value associated with a given key
    ///
    /// May return an arbitrary value if provided an unrecognized key.
    #[must_use]
    fn try_get_mut(&mut self, key: K) -> Option<&mut Self::Value>;

    /// Get a mutable reference to the value associated with a given key
    ///
    /// May panic or return an arbitrary value if provided an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_mut(&mut self, key: K) -> &mut Self::Value {
        self.try_get_mut(key)
            .expect("cannot mutably get unrecognized key")
    }
}

/// A [`Pool`] which does not contain any values, and is always full
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
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
    fn try_remove(&mut self, _key: K) -> Option<Self::Value> {
        None
    }
}

impl<K, V> PoolRef<K> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get(&self, _key: K) -> Option<&Self::Value> {
        None
    }
}

impl<K, V> PoolMut<K> for EmptyPool<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get_mut(&mut self, _key: K) -> Option<&mut Self::Value> {
        None
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
    #[should_panic]
    fn empty_pool_delete_fails() {
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
}
