/*!
Traits for container allocators
*/
use super::*;

pub mod stack;
//TODO: deque
//TODO: set
//TODO: array
//TODO: map
//TODO: iter
//TODO: list

/// A [`Pool`] allocating containers of `Self::Elem`
pub trait ContainerPool<K>: Pool<K> {
    /// The type of items contained in this list
    type Elem;
}