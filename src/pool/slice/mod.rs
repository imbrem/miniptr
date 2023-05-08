/*!
A pool of slices
*/
use crate::index::ContiguousIx;

use super::{container::InsertEmpty, GetMut, GetRef, ObjectPool, Pool};

pub mod free;
use free::*;

/// A pool of slices
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SlicePool<T, F> {
    /// The backing memory of this slice pool
    backing: Vec<T>,
    /// The free lists of this slice pool
    free: F,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
/// A `Vec` composed of indices
pub struct IVec<K> {
    /// The beginning of this vector
    pub begin: K,
    /// The end of this vector
    pub end: K,
    /// This end of this vector's allocation
    pub end_alloc: K,
}

impl<K, T, F> InsertEmpty<IVec<K>> for SlicePool<T, F>
where
    K: ContiguousIx,
{
    fn try_insert_empty(&mut self) -> Result<IVec<K>, ()> {
        Ok(IVec {
            begin: K::new(0),
            end: K::new(0),
            end_alloc: K::new(0),
        })
    }
}

impl<K, T, F> Pool<IVec<K>> for SlicePool<T, F>
where
    F: FreeSlices<[T], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: IVec<K>) {
        self.free
            .dealloc(Slice(key.begin, key.end_alloc), &mut self.backing)
    }
}

impl<K, T, F> ObjectPool<IVec<K>> for SlicePool<T, F>
where
    F: FreeSlices<[T], K>,
{
    type Object = [T];
}

impl<K, T, F> GetRef<IVec<K>, [T]> for SlicePool<T, F>
where
    K: ContiguousIx,
    F: FreeSlices<[T], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_at(&self, key: IVec<K>) -> Option<&[T]> {
        self.backing.get(key.begin.index()..key.end.index())
    }
}

impl<K, T, F> GetMut<IVec<K>, [T]> for SlicePool<T, F>
where
    K: ContiguousIx,
    F: FreeSlices<[T], K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_at_mut(&mut self, key: IVec<K>) -> Option<&mut [T]> {
        self.backing.get_mut(key.begin.index()..key.end.index())
    }
}
