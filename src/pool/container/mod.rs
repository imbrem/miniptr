/*!
Traits for container allocators
*/
use std::collections::VecDeque;

use super::*;

pub mod array;
pub mod map;
pub mod stack;
//TODO: deque
//TODO: set
//TODO: iter
//TODO: list

/// A [`Pool`] allocating containers of `Self::Elem`
pub trait ContainerPool<K>: Pool<K> {
    /// The type of items contained in this list
    type Elem;
}

/// A [`ContainerPool`] implementation which just wraps a pool of [`ContainerLike`]s
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, TransparentWrapper)]
#[repr(transparent)]
pub struct ContainerLikePool<P>(pub P);

/// A trait implemented by things which contain elements of type `Self::Elem`
pub trait Container {
    /// The type of items contained in this container
    type Elem;
}

/// A [`Pool`] which allows the insertion of empty elements
pub trait InsertEmpty<K>: ContainerPool<K> {
    /// Insert an empty container, returning its key
    ///
    /// Returns an error on allocation failure
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn insert_empty(&mut self) -> Result<K, ()> {
        self.insert_empty_with_capacity(0)
    }

    /// Allocate a new, empty stack with the given capacity
    ///
    /// Note that the capacity is *not* guaranteed, and may have a different definition depending on the pool/container type.
    ///
    /// Return an error on failure
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[must_use]
    fn insert_empty_with_capacity(&mut self, capacity: usize) -> Result<K, ()>;

    /// Insert an empty container, returning a unique key
    ///
    /// This means that the returned key is guaranteed to compare disequal to that of any other container which have not been removed.
    /// Behaviour of comparisons with removed keys is unspecified.
    ///
    /// Returns an error on allocation failure. Allocation always fails if the pool does not support this feature
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn insert_unique_empty(&mut self) -> Result<K, ()> {
        Err(())
    }
}

/// A [`Pool`] which associates keys `K` with a length
pub trait LenPool<K>: Pool<K> {
    /// Get the length of the object associated with the key `key`
    #[must_use]
    fn get_len(&self, key: K) -> usize;
}

impl<V> Container for Vec<V> {
    type Elem = V;
}

impl<V> Container for VecDeque<V> {
    type Elem = V;
}