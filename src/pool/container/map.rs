/*!
Traits for map-like containers
*/

use crate::index::ContiguousIx;

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

impl<K, V> GetElem<K, V> for [V] where K: ContiguousIx {
    #[inline(always)]
    fn get_elem(&self, key: K) -> Option<&V> {
        self.get(key.index())
    }
}

impl<K, V> GetElemMut<K, V> for [V] where K: ContiguousIx {
    #[inline(always)]
    fn get_elem_mut(&mut self, key: K) -> Option<&mut V> {
        self.get_mut(key.index())
    }
}