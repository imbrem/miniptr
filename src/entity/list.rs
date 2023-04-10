/*!
Lists backed by a pool
*/

use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bytemuck::TransparentWrapper;

use crate::pool::container::{
    array::{ArrayMutPool, ArrayRefPool, SliceMutPool, SliceRefPool},
    stack::StackPool,
    ContainerPool, InsertEmpty, IsEmptyPool, LenPool,
};

/// A list backed by a pool of type `P`
#[derive(TransparentWrapper)]
#[repr(transparent)]
#[transparent(K)]
pub struct EntityList<T, K, P> {
    ix: K,
    data: PhantomData<(T, P)>,
}

impl<T, K, P> Clone for EntityList<T, K, P>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clone(&self) -> Self {
        Self {
            ix: self.ix.clone(),
            data: PhantomData,
        }
    }
}

impl<T, K, P> Copy for EntityList<T, K, P> where K: Copy {}

impl<T, K, P> Debug for EntityList<T, K, P>
where
    K: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EntityList").field(&self.ix).finish()
    }
}

impl<T, K, P> PartialEq for EntityList<T, K, P>
where
    K: PartialEq,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn eq(&self, other: &Self) -> bool {
        self.ix == other.ix
    }
}

impl<T, K, P> Eq for EntityList<T, K, P> where K: Eq {}

impl<T, K, P> PartialOrd for EntityList<T, K, P>
where
    K: PartialOrd,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ix.partial_cmp(&other.ix)
    }
}

impl<T, K, P> Ord for EntityList<T, K, P>
where
    K: Ord,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ix.cmp(&other.ix)
    }
}

impl<T, K, P> Hash for EntityList<T, K, P>
where
    K: Hash,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ix.hash(state);
    }
}

impl<T, K, P> EntityList<T, K, P>
where
    K: Copy,
    P: ContainerPool<K, Elem = T>,
{
    /// Create a new, empty list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn new(pool: &mut P) -> Self
    where
        P: InsertEmpty<K>,
    {
        EntityList {
            ix: pool.insert_empty(),
            data: PhantomData,
        }
    }

    /// Get the length of a given list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn len(&self, pool: &P) -> usize
    where
        P: LenPool<K>,
    {
        pool.key_len(self.ix)
    }

    /// Return `true` if this list has a length of 0
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn is_empty(&self, pool: &P) -> bool
    where
        P: IsEmptyPool<K>,
    {
        pool.key_is_empty(self.ix)
    }

    /// Push an element to this list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn push(&mut self, item: T, pool: &mut P)
    where
        P: StackPool<K>,
    {
        self.ix = pool.into_pushed(self.ix, item);
    }

    /// Pop an element from this list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn pop(&mut self, pool: &mut P) -> Option<T>
    where
        P: StackPool<K>,
    {
        let (ix, result) = pool.into_popped(self.ix)?;
        self.ix = ix;
        Some(result)
    }

    /// Get a reference to an element in this list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn get<'a>(&self, ix: usize, pool: &'a P) -> Option<&'a T>
    where
        P: ArrayRefPool<K>,
    {
        pool.get_index(self.ix, ix)
    }

    /// Get a mutable reference to an element in this list
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn get_mut<'a>(&self, ix: usize, pool: &'a mut P) -> Option<&'a mut T>
    where
        P: ArrayMutPool<K>,
    {
        pool.get_index_mut(self.ix, ix)
    }

    /// Get this list as a slice
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn as_slice<'a>(&self, pool: &'a P) -> &'a [T]
    where
        P: SliceRefPool<K>,
    {
        pool.slice_at(self.ix)
    }

    /// Get this list as a mutable slice
    #[cfg_attr(not(tarpaulin), inline(always))]
    pub fn as_slice_mut<'a>(&self, pool: &'a mut P) -> &'a mut [T]
    where
        P: SliceMutPool<K>,
    {
        pool.slice_at_mut(self.ix)
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::{pool::slab::SlabPool, slot::DefaultSlot};

    use super::EntityList;

    #[test]
    fn basic_entity_list_usage() {
        let mut pool: SlabPool<DefaultSlot<Vec<u32>>, u32> = SlabPool::new();
        let mut v = EntityList::new(&mut pool);
        assert_eq!(v.len(&pool), 0);
        assert_eq!(v.pop(&mut pool), None);
        assert!(v.is_empty(&pool));
        assert_eq!(v.as_slice(&pool), &[]);
        assert_eq!(v.as_slice_mut(&mut pool), &mut []);
        v.push(3, &mut pool);
        assert_eq!(v.len(&pool), 1);
        assert_eq!(v.get(0, &pool), Some(&3));
        assert_eq!(v.get_mut(0, &mut pool), Some(&mut 3));
        assert_eq!(v.as_slice(&pool), &[3]);
        assert_eq!(v.as_slice_mut(&mut pool), &mut [3]);
        v.as_slice_mut(&mut pool)[0] = 5;
        assert_eq!(v.as_slice(&pool), &[5]);
        assert_eq!(v.as_slice_mut(&mut pool), &mut [5]);
        assert_eq!(v.get(0, &pool), Some(&5));
        assert_eq!(v.get_mut(0, &mut pool), Some(&mut 5));
        assert_eq!(v.pop(&mut pool), Some(5));
        assert_eq!(v.len(&pool), 0);
        assert_eq!(v.as_slice(&pool), &[]);
        assert_eq!(v.as_slice_mut(&mut pool), &mut []);
        assert_eq!(v.pop(&mut pool), None);

        assert_eq!(format!("{v:?}"), "EntityList(0)");
        assert_eq!(v.clone(), v);
        let u = EntityList::new(&mut pool);
        assert_ne!(v, u);
        assert!(v < u);
        assert_eq!(v.partial_cmp(&u), Some(Ordering::Less));
        assert_eq!(v.cmp(&u), Ordering::Less);
    }
}
