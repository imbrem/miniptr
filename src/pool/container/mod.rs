/*!
Traits for container allocators
*/
use super::*;

/// A [`Pool`] allocating containers of `Self::Elem`
pub trait ContainerPool<K>: Pool<K> {
    /// The type of items contained in this list
    type Elem;
}

/// A [`Pool`] allocating stacks containing elements of type `Self::Item`
pub trait StackPool<K>: ContainerPool<K> {
    /// Allocate a new, empty list with the given capacity
    ///
    /// Note that the capacity is *not* guaranteed.
    ///
    /// Return an error on failure
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn new_with_capacity(&mut self, capacity: usize) -> Result<K, ()>;

    /// Push an element to a list
    ///
    /// On success, returns the list's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, panics
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn push(&mut self, key: K, item: Self::Elem) -> K {
        self.try_push(key, item)
            .ok()
            .expect("failed to push to list")
    }

    /// Pop an element from a list
    ///
    /// On success, returns the poppsed value and the list's key, which may have changed.
    /// When called on an empty list, returns `None`, leaving the list unchanged.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop(&mut self, key: K) -> Option<(K, Self::Elem)>;

    /// Try to push an element to a list
    ///
    /// On success, returns the list's key, which may have been changed (in this case, the old key should be considered deleted).
    /// On failure, returns the item, leaving the list unchanged.
    ///
    /// Fails if:
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn try_push(&mut self, key: K, item: Self::Elem) -> Result<K, Self::Elem>;

    /// Pop an element from a list without moving it
    ///
    /// On success, returns the poppsed value and the list's key, which may have changed.
    /// When called on an empty list, returns `Ok(None)`, leaving the list unchanged.
    /// On failure, returns `Err(())`.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop_pinned(&mut self, key: K) -> Result<Option<Self::Elem>, ()>;

    /// Try to push an element to a list without moving it
    ///
    /// On failure, returns the item, leaving the list unchanged
    ///
    /// Fails if:
    /// - Pushing an element to the list would move the list
    /// - The pool is out of capacity
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn push_pinned(&mut self, key: K, item: Self::Elem) -> Result<(), Self::Elem>;

    /// Get the capacity of the list corresponding to the provided key
    ///
    /// The result is guaranteed to be greater than the length of the list.
    /// If a number greater than the length is returned, then it is guaranteed that pushing up to this number of elements to the list will not move it.
    ///
    /// Returns an unspecified value or panics if used on an unrecognized key.
    fn capacity(&self, key: K) -> usize;

    /// Clear the provided list, returning the key to an empty list
    ///
    /// In some implementations, the returned key will preserve the capacity of the input list, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn clear(&mut self, key: K) -> K;

    /// Try to clear the provided list without moving it
    ///
    /// On failure, returns an error, leaving the list unchanged.
    /// Note that this method is allowed to fail where `pop_pinned` in a loop might succeed, since the latter does *not* leave the list unchanged on failure!
    ///
    /// In some implementations, the capacity of the input list will be preserved, but this is *not* guaranteed.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn clear_pinned(&mut self, key: K) -> Result<(), ()>;
}

/// A [`ListPool`] implementation which just wraps a pool of [`StackLike`]s
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, TransparentWrapper)]
#[repr(transparent)]
pub struct StackLikePool<P>(pub P);

/// A trait implemented by things which can be pushed to and popped to like a stack
pub trait StackLike: Default {
    /// The type of items contained in this list
    type Elem;

    /// Allocate a new, empty list with the given capacity
    ///
    /// Note that the capacity is *not* guaranteed.
    ///
    /// Return an error on failure
    fn new_stack_with_capacity(capacity: usize) -> Result<Self, ()>
    where
        Self: Sized;

    /// Push an element to a stack
    ///
    /// Panics if:
    /// - The stack is out of capacity and more cannot be allocated
    fn push_stack(&mut self, item: Self::Elem);

    /// Pop an element from a list
    ///
    /// On success, returns the popped value
    /// When called on an empty list, returns `None`, leaving the list unchanged.
    ///
    /// Leaves the pool in an unspecified state and returns an unspecified value or panics if used on an unrecognized key
    fn pop_stack(&mut self) -> Option<Self::Elem>;

    /// Try to push an element to a list
    ///
    /// On success, returns `Ok(())`
    /// On failure, returns the item, leaving the list unchanged.
    ///
    /// Fails if:
    /// - The list is out of capacity and more cannot be allocated
    fn try_push_stack(&mut self, item: Self::Elem) -> Result<(), Self::Elem>;

    /// Get the capacity of this stack
    fn stack_capacity(&self) -> usize;

    /// Clear the provided stack
    ///
    /// In some implementations, the capacity of the input stack will be preserved, but this is *not* guaranteed
    fn clear_stack(&mut self);
}

impl<V> StackLike for Vec<V> {
    type Elem = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_stack_with_capacity(capacity: usize) -> Result<Self, ()>
    where
        Self: Sized,
    {
        Ok(Vec::with_capacity(capacity))
    }

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

//TODO: this should be a macro...
impl<P, K> Pool<K> for StackLikePool<P>
where
    P: Pool<K>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete(&mut self, key: K) {
        self.0.delete(key)
    }
}

impl<P, K> ObjectPool<K> for StackLikePool<P>
where
    P: ObjectPool<K>,
{
    type Value = P::Value;
}

impl<P, K, V> Insert<K, V> for StackLikePool<P>
where
    P: Insert<K, V>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_insert(&mut self, val: V) -> Result<K, V> {
        self.0.try_insert(val)
    }
}

impl<P, K, V> GetRef<K, V> for StackLikePool<P>
where
    P: GetRef<K, V>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get(&self, key: K) -> Option<&V> {
        self.0.try_get(key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get(&self, key: K) -> &V {
        self.0.get(key)
    }
}

impl<P, K, V> GetMut<K, V> for StackLikePool<P>
where
    P: GetMut<K, V>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_get_mut(&mut self, key: K) -> Option<&mut V> {
        self.0.try_get_mut(key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn get_mut(&mut self, key: K) -> &mut V {
        self.0.get_mut(key)
    }
}

impl<P, K, V> Take<K, V> for StackLikePool<P>
where
    P: Take<K, V>,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_take(&mut self, key: K) -> Option<V> {
        self.0.try_take(key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn take(&mut self, key: K) -> V {
        self.0.take(key)
    }
}

impl<P, K> ContainerPool<K> for StackLikePool<P>
where
    P: InsertPool<K> + PoolMut<K> + PoolRef<K>,
    K: Clone,
    P::Value: StackLike,
{
    type Elem = <P::Value as StackLike>::Elem;
}

impl<P, K> StackPool<K> for StackLikePool<P>
where
    P: InsertPool<K> + PoolMut<K> + PoolRef<K>,
    K: Clone,
    P::Value: StackLike,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_with_capacity(&mut self, capacity: usize) -> Result<K, ()> {
        let stack = P::Value::new_stack_with_capacity(capacity)?;
        self.0.try_insert(stack).map_err(|_| ())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop(&mut self, key: K) -> Option<(K, Self::Elem)> {
        self.0.get_mut(key.clone()).pop_stack().map(|v| (key, v))
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_push(&mut self, key: K, item: Self::Elem) -> Result<K, Self::Elem> {
        self.0
            .get_mut(key.clone())
            .try_push_stack(item)
            .map(|_| key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn pop_pinned(&mut self, key: K) -> Result<Option<Self::Elem>, ()> {
        Ok(self.0.get_mut(key).pop_stack())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn push_pinned(&mut self, key: K, item: Self::Elem) -> Result<(), Self::Elem> {
        self.0.get_mut(key).try_push_stack(item)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity(&self, key: K) -> usize {
        self.0.get(key).stack_capacity()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear(&mut self, key: K) -> K {
        self.0.get_mut(key.clone()).clear_stack();
        key
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn clear_pinned(&mut self, key: K) -> Result<(), ()> {
        self.0.get_mut(key.clone()).clear_stack();
        Ok(())
    }
}
