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

/// A pool which allows the insertion of empty elements
pub trait InsertEmpty<K> {
    /// Allocate an empty container, returning its key
    ///
    /// Panics on allocation failure
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn insert_empty(&mut self) -> K {
        self.try_insert_empty().unwrap()
    }

    /// Allocate an empty container, returning its key
    ///
    /// Returns an error on allocation failure
    #[must_use]
    fn try_insert_empty(&mut self) -> Result<K, ()>;

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

/// A pool which allows the insertion of empty elements with a given capacity
pub trait InsertWithCapacity<K, C = usize> {
    /// Allocate an empty container with the given capacity
    ///
    /// Panics on failure
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn insert_with_capacity(&mut self, capacity: C) -> K {
        self.try_insert_with_capacity(capacity).unwrap()
    }

    /// Allocate an empty container with the given capacity
    ///
    /// Return an error on failure
    #[must_use]
    fn try_insert_with_capacity(&mut self, capacity: C) -> Result<K, ()>;

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
    #[must_use]
    fn new_with_capacity(capacity: C) -> Self;
}

/// A [`Pool`] which associates keys `K` with whether they are empty
pub trait IsEmptyPool<K> {
    /// Get whether the object associated with the key `key` is empty
    #[must_use]
    fn key_is_empty(&self, key: K) -> bool;
}

/// A pool which associates keys `K` with a length
///
/// An empty object should have length 0
pub trait LenPool<K>: IsEmptyPool<K> {
    /// Get the length of the object associated with the key `key`
    #[must_use]
    fn key_len(&self, key: K) -> usize;
}

/// An object which might be empty
pub trait IsEmpty {
    /// Get whether this object might be empty
    #[must_use]
    fn is_empty(&self) -> bool;
}

/// An object with a length
pub trait HasLen: IsEmpty {
    /// Get the length of this object
    #[must_use]
    fn len(&self) -> usize;
}

impl<P, K> IsEmptyPool<K> for P
where
    P: PoolRef<K>,
    P::Object: IsEmpty,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key_is_empty(&self, key: K) -> bool {
        self.at(key).is_empty()
    }
}

impl<P, K> LenPool<K> for P
where
    P: PoolRef<K>,
    P::Object: HasLen,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key_len(&self, key: K) -> usize {
        self.at(key).len()
    }
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
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_with_capacity(capacity: usize) -> Self {
        Vec::with_capacity(capacity)
    }
}

impl<V> WithCapacity for VecDeque<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_with_capacity(capacity: usize) -> Self {
        VecDeque::with_capacity(capacity)
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> WithCapacity for smallvec::SmallVec<A> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_with_capacity(capacity: usize) -> Self {
        smallvec::SmallVec::with_capacity(capacity)
    }
}

#[cfg(feature = "ecow")]
impl<V> WithCapacity for ecow::EcoVec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_with_capacity(capacity: usize) -> Self {
        ecow::EcoVec::with_capacity(capacity)
    }
}

impl<V> IsEmpty for Vec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<V> IsEmpty for [V] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<V, const N: usize> IsEmpty for [V; N] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<V> IsEmpty for VecDeque<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> IsEmpty for smallvec::SmallVec<A> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(feature = "arrayvec")]
impl<V, const N: usize> IsEmpty for arrayvec::ArrayVec<V, N> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(feature = "ecow")]
impl<V> IsEmpty for ecow::EcoVec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<V> HasLen for Vec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<V> HasLen for [V] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<V, const N: usize> HasLen for [V; N] {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        N
    }
}

impl<V> HasLen for VecDeque<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> HasLen for smallvec::SmallVec<A> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "arrayvec")]
impl<V, const N: usize> HasLen for arrayvec::ArrayVec<V, N> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "ecow")]
impl<V> HasLen for ecow::EcoVec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self) -> usize {
        self.len()
    }
}
