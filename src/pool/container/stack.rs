/*!
Traits for containers implementing stacks
*/

use super::*;
use std::collections::VecDeque;

/// A [`Pool`] allocating stacks containing elements of type `Self::Item`
pub trait StackPool<K>: ContainerPool<K> {
    /// Push an element to a stack
    ///
    /// On success, returns the stack's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, panics
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn into_pushed(&mut self, key: K, item: Self::Elem) -> K {
        self.try_into_pushed(key, item)
            .ok()
            .expect("failed to move-push to stack")
    }

    /// Pop an element from a stack, returning a (potentially new) key for the stack as well as the popped value.
    ///
    /// Returns `None` and leaves the stack unchanged given a key for an empty stack.
    /// Otherwise, returns the old value and the new key; the old key (if different from the new key) should be considered deleted.
    /// Panics on failure.
    ///
    /// Fails if:
    /// - The pool is out of capacity, and moving the stack would require an allocation
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn into_popped(&mut self, key: K) -> Option<(K, Self::Elem)> {
        self.try_into_popped(key)
            .expect("failed to move-pop from stack")
    }

    /// Try to pop an element from a stack, returning a (potentially new) key for the stack as well as the popped value.
    ///
    /// Returns `None` and leaves the stack unchanged given a key for an empty stack.
    /// Otherwise, returns the old value and the new key; the old key (if different from the new key) should be considered deleted.
    ///
    /// Fails if:
    /// - The pool is out of capacity, and moving the stack would require an allocation
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_into_popped(&mut self, key: K) -> Result<Option<(K, Self::Elem)>, ()>;

    /// Try to push an element to a stack, returning a (potentially new) key for the stack as well as the popped value
    ///
    /// On success, returns the stack's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, returns the item, leaving the stack unchanged.
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_into_pushed(&mut self, key: K, item: Self::Elem) -> Result<K, Self::Elem>;

    /// Pop an element from a stack
    ///
    /// On success, returns the popped value.
    /// When called on an empty stack, returns `None`, leaving the stack unchanged.
    /// Panics on failure.
    ///
    /// Fails if:
    /// - Popping an element to the stack would require moving the stack
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop(&mut self, key: K) -> Option<Self::Elem> {
        self.try_pop(key).expect("failed to pop from stack")
    }

    /// Push an element to a stack
    ///
    /// Panics on failure
    ///
    /// Fails if:
    /// - The pool is out of capacity
    /// - Pushing an element to the stack would require moving the stack
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push(&mut self, key: K, item: Self::Elem) {
        self.try_push(key, item)
            .ok()
            .expect("failed to push from stack")
    }

    /// Try to pop an element from a stack
    ///
    /// On success, returns the popped value.
    /// When called on an empty stack, returns `Ok(None)`, leaving the stack unchanged.
    /// On failure, returns `Err(())`.
    ///
    /// Fails if:
    /// - Popping an element to the stack would require moving the stack
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_pop(&mut self, key: K) -> Result<Option<Self::Elem>, ()>;

    /// Try to push an element to a stack
    ///
    /// On failure, returns the item, leaving the stack unchanged
    ///
    /// Fails if:
    /// - The pool is out of capacity
    /// - Pushing an element to the stack would require moving the stack
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_push(&mut self, key: K, item: Self::Elem) -> Result<(), Self::Elem>;

    /// Get the capacity of the stack corresponding to the provided key
    ///
    /// If a number greater than the length is returned, then it is guaranteed that pushing up to this number of elements to the stack will always succeed.
    /// If a number less than or equal to the length is returned, then no guarantees are made; in particular, 0 is always a safe return value.
    ///
    /// Returns an unspecified value or panics if used on an unrecognized key.
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn key_capacity(&self, _key: K) -> usize {
        0
    }

    /// Clear the provided stack, returning the key to an empty stack
    ///
    /// In some implementations, the returned key will preserve the capacity of the input stack, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn cleared_key(&mut self, key: K) -> K;

    /// Try to clear the provided stack without moving it
    ///
    /// On failure, returns an error, leaving the stack unchanged.
    /// Note that this method is allowed to fail where `pop_pinned` in a loop might succeed, since the latter does *not* leave the stack unchanged on failure!
    ///
    /// In some implementations, the capacity of the input stack will be preserved, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn clear_key(&mut self, key: K) -> Result<(), ()>;
}

/// A trait implemented by things which can be pushed to and popped to like a stack
pub trait StackLike: Container + Default {
    /// Push an element to a stack
    ///
    /// Panics if:
    /// - The stack is out of capacity and more cannot be allocated
    fn push_stack(&mut self, item: Self::Elem);

    /// Pop an element from a stack
    ///
    /// On success, returns the popped value
    /// When called on an empty stack, returns `None`, leaving the stack unchanged.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop_stack(&mut self) -> Option<Self::Elem>;

    /// Try to push an element to a stack
    ///
    /// On success, returns `Ok(())`
    /// On failure, returns the item, leaving the stack unchanged.
    ///
    /// Fails if:
    /// - The stack is out of capacity and more cannot be allocated
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem>;

    /// Get the capacity of this stack
    fn stack_capacity(&self) -> usize;

    /// Clear the provided stack
    ///
    /// In some implementations, the capacity of the input stack will be preserved, but this is *not* guaranteed
    fn clear_stack(&mut self);
}

impl<P, K> StackPool<K> for P
where
    P: InsertPool<K> + PoolMut<K> + PoolRef<K>,
    K: Clone,
    P::Object: StackLike,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_popped(&mut self, key: K) -> Result<Option<(K, Self::Elem)>, ()> {
        Ok(self.at_mut(key.clone()).pop_stack().map(|v| (key, v)))
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_pushed(&mut self, key: K, item: Self::Elem) -> Result<K, Self::Elem> {
        self.at_mut(key.clone()).try_push_stack(item).map(|_| key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_pop(&mut self, key: K) -> Result<Option<Self::Elem>, ()> {
        Ok(self.at_mut(key).pop_stack())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push(&mut self, key: K, item: Self::Elem) -> Result<(), Self::Elem> {
        self.at_mut(key).try_push_stack(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key_capacity(&self, key: K) -> usize {
        self.at(key).stack_capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn cleared_key(&mut self, key: K) -> K {
        self.at_mut(key.clone()).clear_stack();
        key
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_key(&mut self, key: K) -> Result<(), ()> {
        self.at_mut(key.clone()).clear_stack();
        Ok(())
    }
}

impl<V> StackLike for Vec<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_stack(&mut self, item: Self::Elem) {
        self.push(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_stack(&mut self) -> Option<Self::Elem> {
        self.pop()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem> {
        self.push(item);
        Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stack_capacity(&self) -> usize {
        self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_stack(&mut self) {
        self.clear()
    }
}

impl<V> StackLike for VecDeque<V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_stack(&mut self, item: Self::Elem) {
        self.push_back(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_stack(&mut self) -> Option<Self::Elem> {
        self.pop_back()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem> {
        self.push_back(item);
        Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stack_capacity(&self) -> usize {
        self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_stack(&mut self) {
        self.clear()
    }
}

#[cfg(feature = "smallvec")]
impl<A> StackLike for smallvec::SmallVec<A>
where
    A: smallvec::Array,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_stack(&mut self, item: Self::Elem) {
        self.push(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_stack(&mut self) -> Option<Self::Elem> {
        self.pop()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem> {
        self.push(item);
        Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stack_capacity(&self) -> usize {
        self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_stack(&mut self) {
        self.clear()
    }
}

#[cfg(feature = "arrayvec")]
impl<V, const N: usize> StackLike for arrayvec::ArrayVec<V, N> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_stack(&mut self, item: Self::Elem) {
        self.push(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_stack(&mut self) -> Option<Self::Elem> {
        self.pop()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem> {
        self.push(item);
        Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stack_capacity(&self) -> usize {
        self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_stack(&mut self) {
        self.clear()
    }
}

#[cfg(feature = "ecow")]
impl<V> StackLike for ecow::EcoVec<V>
where
    V: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_stack(&mut self, item: Self::Elem) {
        self.push(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_stack(&mut self) -> Option<Self::Elem> {
        self.pop()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem> {
        self.push(item);
        Ok(())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn stack_capacity(&self) -> usize {
        self.capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_stack(&mut self) {
        self.clear()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn vec_like_test(
        stack: &mut (impl Container<Elem = u32>
                  + StackLike
                  + IsEmpty
                  + HasLen
                  + GetRef<usize, u32>
                  + GetMut<usize, u32>),
    ) {
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        stack.push_stack(4);
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
        stack.push_stack(3);
        stack.push_stack(2);
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop_stack(), Some(2));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.pop_stack(), Some(3));
        stack.push_stack(7);
        stack.push_stack(8);
        assert_eq!(stack.pop_stack(), Some(8));
        assert_eq!(stack.pop_stack(), Some(7));
        assert_eq!(stack.pop_stack(), Some(4));
        assert!(stack.is_empty());
        assert_eq!(stack.pop_stack(), None);
        assert!(stack.is_empty());
        stack.push_stack(3);
        stack.push_stack(2);
        assert_eq!(stack.pop_stack(), Some(2));
        assert!(!stack.is_empty());
        stack.clear_stack();
        assert!(stack.is_empty());
        assert_eq!(stack.pop_stack(), None);
        assert_eq!(stack.pop_stack(), None);
        stack.clear_stack();
        assert_eq!(stack.pop_stack(), None);
        assert!(stack.is_empty());
        assert_eq!(stack.try_push_stack(8), Ok(()));
        assert!(!stack.is_empty());
        let _ = stack.stack_capacity();

        assert_eq!(stack.at(0), &8);
        assert_eq!(stack.at_mut(0), &mut 8);
        assert_eq!(stack.try_at(0), Some(&8));
        assert_eq!(stack.try_at_mut(0), Some(&mut 8));
        assert_eq!(stack.try_at(1), None);
        assert_eq!(stack.try_at_mut(1), None);
    }

    #[test]
    fn default_stack_like() {
        vec_like_test(&mut Vec::new_with_capacity(12));
        vec_like_test(&mut VecDeque::new_with_capacity(34));
        #[cfg(feature = "smallvec")]
        vec_like_test(&mut smallvec::SmallVec::<[u32; 2]>::new_with_capacity(6));
        #[cfg(feature = "arrayvec")]
        vec_like_test(&mut arrayvec::ArrayVec::<u32, 10>::new());
        #[cfg(feature = "ecow")]
        vec_like_test(&mut ecow::EcoVec::new_with_capacity(4));
    }
}
