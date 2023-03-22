/*!
Traits for container allocators
*/
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

/// A [`Pool`] which allows the insertion of empty elements
pub trait InsertEmpty<K>: ContainerPool<K> {
    /// Insert an empty container, returning its key
    /// 
    /// Returns an error on allocation failure
    #[must_use]
    fn insert_empty(&mut self) -> Result<K, ()>;
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