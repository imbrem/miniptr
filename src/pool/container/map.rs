/*!
Traits for map-like containers
*/

use std::collections::VecDeque;

use crate::{
    index::ContiguousIx,
    pool::{PoolMut, PoolRef},
};

/// Given a key `K` an index `I`, get a reference to the associated value `V`
pub trait GetIndex<K, I, V> {
    #[must_use]
    fn get_index(&self, key: K, elem: I) -> Option<&V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_index_unchecked(&self, key: K, elem: I) -> Option<&V> {
        self.get_index(key, elem)
    }
}

/// Given a key `K` and an index `I`, get a mutable reference to the associated value `V`
pub trait GetIndexMut<K, I, V> {
    #[must_use]
    fn get_index_mut(&mut self, key: K, elem: I) -> Option<&mut V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_index_mut_unchecked(&mut self, key: K, elem: I) -> Option<&mut V> {
        self.get_index_mut(key, elem)
    }
}

/// Given a key `K` get a reference to the associated element `V`
pub trait GetElem<K, V> {
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_unchecked(&self, key: K) -> Option<&V> {
        self.get_elem(key)
    }
}

impl<P, K, I, V> GetIndex<K, I, V> for P
where
    P: PoolRef<K>,
    P::Object: GetElem<I, V> + 'static, //TODO: relax this?
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index(&self, key: K, elem: I) -> Option<&V> {
        self.try_get_value(key)?.get_elem(elem)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_unchecked(&self, key: K, elem: I) -> Option<&V> {
        self.get_value(key).get_elem_unchecked(elem)
    }
}

impl<P, K, I, V> GetIndexMut<K, I, V> for P
where
    P: PoolMut<K>,
    P::Object: GetElemMut<I, V> + 'static, //TODO: relax this?
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_mut(&mut self, key: K, elem: I) -> Option<&mut V> {
        self.try_get_value_mut(key)?.get_elem_mut(elem)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_mut_unchecked(&mut self, key: K, elem: I) -> Option<&mut V> {
        self.get_value_mut(key).get_elem_mut_unchecked(elem)
    }
}

/// Given a key `K` get a mutable reference to the associated element `V`
pub trait GetElemMut<K, V> {
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut_unchecked(&mut self, key: K) -> Option<&mut V> {
        self.get_elem_mut(key)
    }
}

impl<K, V> GetElem<K, V> for [V]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetElemMut<K, V> for [V]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

impl<K, V> GetElem<K, V> for Vec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetElemMut<K, V> for Vec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

impl<K, V> GetElem<K, V> for VecDeque<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetElemMut<K, V> for VecDeque<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

#[cfg(feature = "smallvec")]
impl<K, A: smallvec::Array> GetElem<K, A::Item> for smallvec::SmallVec<A>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&A::Item> {
        self.get(key.index())
    }
}

#[cfg(feature = "smallvec")]
impl<K, A: smallvec::Array> GetElemMut<K, A::Item> for smallvec::SmallVec<A>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut A::Item> {
        self.get_mut(key.index())
    }
}

#[cfg(feature = "ecow")]
impl<K, V> GetElem<K, V> for ecow::EcoVec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

#[cfg(feature = "ecow")]
impl<K, V> GetElemMut<K, V> for ecow::EcoVec<V>
where
    K: ContiguousIx,
    V: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V> {
        self.make_mut().get_mut(key.index())
    }
}
