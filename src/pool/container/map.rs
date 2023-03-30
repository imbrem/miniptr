/*!
Traits for map-like containers
*/

use std::collections::VecDeque;

use crate::{
    index::ContiguousIx,
    pool::{GetMut, GetRef, PoolMut, PoolRef},
};

/// Given a key `K` an index `I`, get a reference to the associated value `V`
pub trait GetIndex<K, I, V> {
    #[must_use]
    fn get_index(&self, key: K, elem: I) -> Option<&V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_index_unchecked(&self, key: K, elem: I) -> &V {
        self.get_index(key, elem).expect("invalid key")
    }
}

/// Given a key `K` and an index `I`, get a mutable reference to the associated value `V`
pub trait GetIndexMut<K, I, V> {
    #[must_use]
    fn get_index_mut(&mut self, key: K, elem: I) -> Option<&mut V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_index_mut_unchecked(&mut self, key: K, elem: I) -> &mut V {
        self.get_index_mut(key, elem).expect("invalid key")
    }
}

impl<P, K, I, V> GetIndex<K, I, V> for P
where
    P: PoolRef<K>,
    P::Object: GetRef<I, V> + 'static, //TODO: relax this?
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index(&self, key: K, elem: I) -> Option<&V> {
        self.try_get_value(key)?.try_at(elem)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_unchecked(&self, key: K, elem: I) -> &V {
        self.get_value(key).at(elem)
    }
}

impl<P, K, I, V> GetIndexMut<K, I, V> for P
where
    P: PoolMut<K>,
    P::Object: GetMut<I, V> + 'static, //TODO: relax this?
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_mut(&mut self, key: K, elem: I) -> Option<&mut V> {
        self.try_get_value_mut(key)?.try_at_mut(elem)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_index_mut_unchecked(&mut self, key: K, elem: I) -> &mut V {
        self.get_value_mut(key).at_mut(elem)
    }
}

impl<K, V> GetRef<K, V> for [V]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetMut<K, V> for [V]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

impl<K, V, const N: usize> GetRef<K, V> for [V; N]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V, const N: usize> GetMut<K, V> for [V; N]
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

impl<K, V> GetRef<K, V> for Vec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetMut<K, V> for Vec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

impl<K, V> GetRef<K, V> for VecDeque<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetMut<K, V> for VecDeque<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

#[cfg(feature = "smallvec")]
impl<K, A: smallvec::Array> GetRef<K, A::Item> for smallvec::SmallVec<A>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&A::Item> {
        self.get(key.index())
    }
}

#[cfg(feature = "smallvec")]
impl<K, A: smallvec::Array> GetMut<K, A::Item> for smallvec::SmallVec<A>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut A::Item> {
        self.get_mut(key.index())
    }
}

#[cfg(feature = "arrayvec")]
impl<K, V, const N: usize> GetRef<K, V> for arrayvec::ArrayVec<V, N>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

#[cfg(feature = "arrayvec")]
impl<K, V, const N: usize> GetMut<K, V> for arrayvec::ArrayVec<V, N>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}

#[cfg(feature = "ecow")]
impl<K, V> GetRef<K, V> for ecow::EcoVec<V>
where
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

#[cfg(feature = "ecow")]
impl<K, V> GetMut<K, V> for ecow::EcoVec<V>
where
    K: ContiguousIx,
    V: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_at_mut(&mut self, key: K) -> Option<&mut V> {
        self.make_mut().get_mut(key.index())
    }
}
