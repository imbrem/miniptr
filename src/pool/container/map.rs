/*!
Traits for map-like containers
*/

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
    fn get_index_mut(&self, key: K, elem: I) -> Option<&mut V>;
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn get_index_mut_unchecked(&self, key: K, elem: I) -> Option<&mut V> {
        self.get_index_mut(key, elem)
    }
}
