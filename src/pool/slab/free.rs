/*!
A free list implementation for a slab allocator
*/
use crate::{
    index::ContiguousIx,
    slot::{KeySlot, RemoveSlot},
};

/// A free list implementation over a backing of slots
pub trait FreeList<B: ?Sized, K> {
    /// Allocate a slot in the backing, returning it's index
    ///
    /// Returns `None` on failure
    #[must_use]
    fn alloc(&mut self, backing: &mut B) -> Option<K>;

    /// Deallocate a slot in the backing, putting it on the free list
    ///
    /// If the slot has not been previously alloc'ed or placed into the backing as a valid key, the behaviour is unspecified.
    fn delete(&mut self, key: K, backing: &mut B);

    /// Clear this free list, resetting it
    fn clear(&mut self, backing: &mut B);
}

/// A free list implementation which allows the removal of values
pub trait RemovalList<B: ?Sized, K>: FreeList<B, K> {
    type Value;

    /// Deallocate a slot in the backing, putting it on the free list and returning it's value
    ///
    /// If the slot has not been previously alloc'ed or placed into the backing as a valid key, the behaviour is unspecified.
    #[must_use]
    fn try_remove(&mut self, key: K, backing: &mut B) -> Option<Self::Value>;

    /// Deallocate a slot in the backing, putting it on the free list and returning it's value
    ///
    /// If the slot has not been previously alloc'ed or placed into the backing as a valid key, the behaviour is unspecified.
    #[must_use]
    fn remove(&mut self, key: K, backing: &mut B) -> Self::Value {
        self.try_remove(key, backing).expect("remove to succeed")
    }
}

/// A free list which supports querying the next element of the list
pub trait NextFreeList<B: ?Sized, K>: FreeList<B, K> {
    /// Get the next free slot in the list
    ///
    /// The returned index should be free, and the next call to `alloc` should return this index unless the free list or backing have been modified in the meantime
    #[must_use]
    fn next_free(&self, backing: &B) -> Option<K>;
}

/// A free list which supports querying it's capacity
pub trait FreeListCapacity<B: ?Sized, K>: FreeList<B, K> {
    /// Get the number of slots in this free list
    #[must_use]
    fn len(&self, backing: &B) -> usize;
}

/// A simple free list consisting of a vector of free keys
///
/// Returns the most recently free'd key first, to maximize caching
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct KeyList<K>(pub Vec<K>);

impl<K> Default for KeyList<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<S, K> FreeList<[S], K> for KeyList<K>
where
    S: RemoveSlot,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn alloc(&mut self, _backing: &mut [S]) -> Option<K> {
        self.0.pop()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K, backing: &mut [S]) {
        if let Some(slot) = backing.get_mut(key.index()) {
            slot.delete_value();
            self.0.push(key);
        }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear(&mut self, _backing: &mut [S]) {
        self.0.clear()
    }
}

impl<S, K> RemovalList<[S], K> for KeyList<K>
where
    S: RemoveSlot,
    K: ContiguousIx,
{
    type Value = S::Value;

    fn try_remove(&mut self, key: K, backing: &mut [S]) -> Option<S::Value> {
        let value = backing.get_mut(key.index())?.try_remove_value()?;
        self.0.push(key);
        Some(value)
    }

    fn remove(&mut self, key: K, backing: &mut [S]) -> S::Value {
        let value = backing[key.index()].remove_value();
        self.0.push(key);
        value
    }
}

impl<S, K> NextFreeList<[S], K> for KeyList<K>
where
    S: RemoveSlot,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn next_free(&self, _backing: &[S]) -> Option<K> {
        self.0.last().cloned()
    }
}

impl<S, K> FreeListCapacity<[S], K> for KeyList<K>
where
    S: RemoveSlot,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self, _backing: &[S]) -> usize {
        self.0.len()
    }
}

/// An intrusive free list, with keys of type `K`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct IntrusiveFree {
    free_head: usize,
}

impl Default for IntrusiveFree {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn default() -> Self {
        Self {
            free_head: usize::MAX,
        }
    }
}

impl<S, K> FreeList<[S], K> for IntrusiveFree
where
    S: KeySlot<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn alloc(&mut self, backing: &mut [S]) -> Option<K> {
        let slot = backing.get_mut(self.free_head)?;
        let key = slot.key().index();
        let old = self.free_head;
        self.free_head = if key == self.free_head {
            usize::MAX
        } else {
            key
        };
        Some(K::new_unchecked(old))
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K, backing: &mut [S]) {
        let ix = key.index();
        if let Some(slot) = backing.get_mut(ix) {
            slot.set_key(K::try_new(self.free_head).unwrap_or(key));
            self.free_head = ix;
        }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear(&mut self, _backing: &mut [S]) {
        self.free_head = usize::MAX
    }
}

impl<S, K> RemovalList<[S], K> for IntrusiveFree
where
    S: KeySlot<K>,
    K: ContiguousIx,
{
    type Value = S::Value;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove(&mut self, key: K, backing: &mut [S]) -> Option<S::Value> {
        let ix = key.index();
        let value = backing
            .get_mut(ix)?
            .try_swap_key(K::try_new(self.free_head).unwrap_or(key))?;
        self.free_head = ix;
        Some(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn remove(&mut self, key: K, backing: &mut [S]) -> S::Value {
        let ix = key.index();
        let value = backing
            .get_mut(ix)
            .expect("key to be valid")
            .swap_key(K::try_new(self.free_head).unwrap_or(key));
        self.free_head = ix;
        value
    }
}

impl<S, K> NextFreeList<[S], K> for IntrusiveFree
where
    S: KeySlot<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn next_free(&self, _backing: &[S]) -> Option<K> {
        K::try_new(self.free_head)
    }
}

impl<S, K> FreeListCapacity<[S], K> for IntrusiveFree
where
    S: KeySlot<K>,
    K: ContiguousIx,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn len(&self, backing: &[S]) -> usize {
        let mut len = 0;
        let mut curr = self.free_head;
        while let Some(slot) = backing.get(curr) {
            len += 1;
            let key = slot.key().index();
            if key == curr {
                break;
            }
            curr = key
        }
        len
    }
}
