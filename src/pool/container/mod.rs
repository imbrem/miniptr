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

impl<P, K> ContainerPool<K> for P
where
    P: ObjectPool<K>,
    P::Object: Container,
{
    type Elem = <P::Object as Container>::Elem;
}

/// A trait implemented by things which contain elements of type `Self::Elem`
pub trait Container {
    /// The type of items contained in this container
    type Elem;
}

/// A [`Pool`] which allows the insertion of empty elements
pub trait InsertEmpty<K>: ContainerPool<K> {
    /// Allocate an empty container, returning its key
    ///
    /// Returns an error on allocation failure
    #[must_use]
    fn insert_empty(&mut self) -> Result<K, ()>;

    /// Allocate an empty container, returning a unique key
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

/// A [`Pool`] which allows the insertion of empty elements with a given capacity
pub trait InsertWithCapacity<K, C = usize>: ContainerPool<K> {
    /// Allocate an empty container with the given capacity
    ///
    /// Note that the capacity is *not* guaranteed, and may have a different definition depending on the pool/container type.
    ///
    /// Return an error on failure
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[must_use]
    fn insert_with_capacity(&mut self, capacity: C) -> Result<K, ()>;

    /// Allocate an empty container with the given capacity, returning a unique key
    ///
    /// This means that the returned key is guaranteed to compare disequal to that of any other container which have not been removed.
    /// Behaviour of comparisons with removed keys is unspecified.
    ///
    /// Returns an error on allocation failure. Allocation always fails if the pool does not support this feature
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn insert_unique_with_capacity(&mut self, _capacity: C) -> Result<K, ()> {
        Err(())
    }
}

/// An object which can be created with a given capacity
pub trait WithCapacity<C = usize> {
    fn new_with_capacity(capacity: C) -> Self;
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

impl<V> Container for [V] {
    type Elem = V;
}

impl<V, const N: usize> Container for [V; N] {
    type Elem = V;
}

impl<V> Container for VecDeque<V> {
    type Elem = V;
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> Container for smallvec::SmallVec<A> {
    type Elem = A::Item;
}

#[cfg(feature = "arrayvec")]
impl<V, const N: usize> Container for arrayvec::ArrayVec<V, N> {
    type Elem = V;
}

#[cfg(feature = "ecow")]
impl<V> Container for ecow::EcoVec<V> {
    type Elem = V;
}

impl<V> WithCapacity for Vec<V> {
    fn new_with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

impl<V> WithCapacity for VecDeque<V> {
    fn new_with_capacity(capacity: usize) -> Self {
        VecDeque::with_capacity(capacity)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> WithCapacity for smallvec::SmallVec<A> {
    fn new_with_capacity(capacity: usize) -> Self {
        smallvec::SmallVec::with_capacity(capacity)
    }
}

#[cfg(feature = "ecow")]
impl<V> WithCapacity for ecow::EcoVec<V> {
    fn new_with_capacity(capacity: usize) -> Self {
        ecow::EcoVec::with_capacity(capacity)
    }
}
