/*!
A trait for list allocators
*/
use super::*;

/// A [`Pool`] allocating lists of type `Self::Item`
pub trait ListPool<K, V>: Pool<K, Value = [V]> {
    type Item;

    /// Allocate a new, empty list with the given capacity
    ///
    /// Note that the capacity is *not* guaranteed.
    ///
    /// Return an error on failure
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn new_with_capacity(&mut self, capacity: usize) -> Result<K, ()>;

    /// Push an element to a list
    ///
    /// On success, returns the list's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, panics
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn push(&mut self, key: K, item: Self::Item) -> K {
        self.try_push(key, item)
            .ok()
            .expect("failed to push to list")
    }

    /// Pop an element from a list
    ///
    /// On success, returns the poppsed value and the list's key, which may have changed.
    /// When called on an empty list, returns `None`, leaving the list unchanged.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop(&mut self, key: K) -> Option<(K, Self::Item)>;

    /// Try to push an element to a list
    ///
    /// On success, returns the list's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, returns the item, leaving the list unchanged.
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_push(&mut self, key: K, item: Self::Item) -> Result<K, Self::Item>;

    /// Pop an element from a list without moving it
    ///
    /// On success, returns the poppsed value and the list's key, which may have changed.
    /// When called on an empty list, returns `Ok(None)`, leaving the list unchanged.
    /// On failure, returns `Err(())`.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop_pinned(&mut self, key: K, item: Self::Item) -> Result<Option<Self::Item>, ()>;

    /// Try to push an element to a list without moving it
    ///
    /// On failure, returns the item, leaving the list unchanged
    ///
    /// Fails if:
    /// - Pushing an element to the list would move the list
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn push_pinned(&mut self, key: K, item: Self::Item) -> Result<(), Self::Item>;

    /// Get the length of the list corresponding to the provided key
    ///
    /// Returns an unspecified value or panics if used on an unrecognized key.
    fn len(&self, key: K) -> usize;

    /// Get the capacity of the list corresponding to the provided key
    ///
    /// The result is guaranteed to be greater than the length of the list.
    /// If a number greater than the length is returned, then it is guaranteed that pushing up to this number of elements to the list will not move it.
    ///
    /// Returns an unspecified value or panics if used on an unrecognized key.
    fn capacity(&self, key: K) -> usize;

    /// Clear the provided list, returning the key to an empty list
    ///
    /// In some implementations, the returned key will preserve the capacity of the input list, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn clear(&mut self, key: K) -> K;

    /// Try to clear the provided list without moving it
    ///
    /// On failure, returns an error, leaving the list unchanged.
    /// Note that this method is allowed to fail where `pop_pinned` in a loop might succeed, since the latter does *not* leave the list unchanged on failure!
    ///
    /// In some implementations, the capacity of the input list will be preserved, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn clear_pinned(&mut self, key: K) -> Result<(), ()>;
}