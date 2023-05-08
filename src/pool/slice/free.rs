/*!
A free list implementation for a slice allocator
*/
use crate::{index::ContiguousIx, slot::KeySlot};

/// A free list of slices indexed by capacity
pub trait FreeSlices<B: ?Sized, K> {
    /// Allocate a slice in the backing, returning it's index
    ///
    /// Returns `None` on failure
    #[must_use]
    fn alloc(&mut self, capacity: usize, backing: &mut B) -> Option<Slice<K>>;

    /// Deallocate a slice in the backing, putting it on the free list, potentially with a different capacity
    ///
    /// If the slot has not been previously alloc'ed or placed into the backing as a valid key, the behaviour is unspecified.
    fn dealloc(&mut self, alloc: Slice<K>, backing: &mut B);

    /// Clear this free list, resetting it
    fn clear(&mut self, backing: &mut B);
}

/// A slice composed of indices
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Slice<K>(pub K, pub K);

/// An intrusive list for each size class
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct IntrusiveClasses<S> {
    free_heads: Vec<usize>,
    size_classes: S,
}

impl<S> IntrusiveClasses<S>
where
    S: SizeClasses,
{
    #[inline]
    pub fn alloc_size_class<K, T>(&mut self, size_class: u32, backing: &mut [T]) -> Option<Slice<K>>
    where
        K: ContiguousIx,
        T: KeySlot<K>,
    {
        if size_class == 0 {
            return None;
        }
        let capacity = self.size_classes.capacity(size_class);
        let (next_index, free_head) =
            if let Some(slot) = backing.get(*self.free_heads.get(size_class as usize - 1)?) {
                (slot.key().index(), self.free_heads[size_class as usize - 1])
            } else {
                debug_assert_eq!(self.free_heads[size_class as usize - 1], usize::MAX);
                let upper_class = self.size_classes.split_size_class(size_class)?;
                let size_class_alloc = self.alloc_size_class(upper_class, backing)?;
                let begin = size_class_alloc.0.index();
                let slack = Slice(K::new_unchecked(begin + capacity), size_class_alloc.1);
                self.dealloc(slack, backing);
                (self.free_heads[size_class as usize - 1], begin)
            };
        if next_index == free_head {
            if size_class as usize == self.free_heads.len() {
                self.free_heads.pop();
            } else {
                self.free_heads[size_class as usize - 1] = usize::MAX
            }
        } else {
            self.free_heads[size_class as usize - 1] = next_index
        };
        let result = Some(Slice(
            K::new_unchecked(free_head),
            K::new_unchecked(free_head + capacity),
        ));
        result
    }
}

impl<K, S, T> FreeSlices<[T], K> for IntrusiveClasses<S>
where
    K: ContiguousIx,
    S: SizeClasses,
    T: KeySlot<K>,
{
    #[inline]
    fn alloc(&mut self, capacity: usize, backing: &mut [T]) -> Option<Slice<K>> {
        let size_class = self.size_classes.size_class_containing(capacity);
        self.alloc_size_class(size_class, backing)
    }

    #[inline]
    fn dealloc(&mut self, alloc: Slice<K>, backing: &mut [T]) {
        let begin = alloc.0.index();
        let end = alloc.1.index();
        let size_class = self.size_classes.size_class_contained(end - begin);
        if size_class == 0 {
            return; //TODO: optimize
        }
        let new_len = size_class as usize;
        if self.free_heads.len() < new_len {
            self.free_heads.resize(new_len, usize::MAX);
        }
        let old_free_head = self.free_heads[size_class as usize - 1];
        let new_free_head = alloc.0.index();
        backing[new_free_head].set_key(K::try_new(old_free_head).unwrap_or(alloc.0));
        self.free_heads[size_class as usize - 1] = new_free_head;
        let begin_slack = begin + self.size_classes.capacity(size_class);
        let slack = Slice(K::new_unchecked(begin_slack), alloc.1);
        self.dealloc(slack, backing)
    }

    #[inline]
    fn clear(&mut self, _backing: &mut [T]) {
        self.free_heads.clear()
    }
}

/// A mapping from capacities to size classes
pub trait SizeClasses {
    /// Get the index of the smallest size class containing this capacity
    ///
    /// Returns `0` for capacity `0`
    /// Returns `u32::MAX` if there is no matching size class
    fn size_class_containing(&self, capacity: usize) -> u32;
    /// Get the index of the largest size class contained within this capacity
    ///
    /// Returns `0` for capacity `0`
    fn size_class_contained(&self, capacity: usize) -> u32;
    /// Get the index of the size class corresponding *exactly* to this capacity
    ///
    /// The return value is unspecified if `capacity` is not a size class
    #[inline]
    fn size_class_exact(&self, capacity: usize) -> u32 {
        self.size_class_containing(capacity)
    }
    /// Get the capacity of a given size class
    ///
    /// Returns an unspecified value if `size_class` is out of bounds
    fn capacity(&self, size_class: u32) -> usize;
    /// Round up a capacity to the nearest size class
    ///
    /// The result must be:
    /// - Greater than or equal to `capacity`
    /// - Equal to `self.capacity(self.size_class_containing(capacity))`
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn round_up_capacity(&self, capacity: usize) -> usize {
        self.capacity(self.size_class_containing(capacity))
    }
    /// Round down a capacity to the nearest size class
    ///
    /// The result must be:
    /// - Less than or equal to `capacity`
    /// - Equal to `self.capacity(self.size_class_contained(capacity))`
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn round_down_capacity(&self, capacity: usize) -> usize {
        self.capacity(self.size_class_contained(capacity))
    }
    /// Return the smallest size class that can be split into efficiently into this size class
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn split_size_class(&self, _size_class: u32) -> Option<u32> {
        None
    }
}

/// Constant exponential size classes where `ExpSize<N, B>.capacity(n) = 2**(n*N + B)`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct Exp2Size<const N: usize, const B: usize>;

impl<const N: usize, const B: usize> SizeClasses for Exp2Size<N, B> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn size_class_containing(&self, capacity: usize) -> u32 {
        //TODO: optimize
        let contained = self.size_class_contained(capacity);
        contained + (self.capacity(contained) < capacity) as u32
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn size_class_contained(&self, capacity: usize) -> u32 {
        //TODO: optimize
        if capacity < (1usize << B) {
            return 0;
        }
        1 + capacity.ilog2().saturating_sub(B as u32) / N as u32
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity(&self, size_class: u32) -> usize {
        if size_class == 0 {
            0
        } else {
            1usize << (B as u32 + (size_class - 1) * N as u32)
        }
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn split_size_class(&self, size_class: u32) -> Option<u32> {
        Some(size_class + 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn free_list_alloc() {
        let mut classes = IntrusiveClasses::<Exp2Size<1, 2>>::default();
        let mut backing = [0; 1024];
        assert_eq!(classes.alloc(0, &mut backing), None::<Slice<u32>>);
        assert_eq!(classes.alloc(4, &mut backing), None::<Slice<u32>>);
        classes.dealloc(Slice(0, 4), &mut backing);
        assert_eq!(classes.alloc(8, &mut backing), None::<Slice<u32>>);
        assert_eq!(classes.alloc(2, &mut backing), Some(Slice(0, 4)));
        assert_eq!(classes.alloc(4, &mut backing), None::<Slice<u32>>);
        classes.dealloc(Slice(0, 7), &mut backing); //Note: memory in 4..7 is leaked, since it can't fit into the smallest size class
        classes.dealloc(Slice(8, 12), &mut backing);
        classes.dealloc(Slice(12, 24), &mut backing); //Note: memory in 20..24 is *not* leaked, since it fits in a smaller size class
        assert_eq!(classes.alloc(2, &mut backing), Some(Slice(20, 24)));
        assert_eq!(classes.alloc(2, &mut backing), Some(Slice(8, 12)));
        assert_eq!(classes.alloc(3, &mut backing), Some(Slice(0, 4)));
        assert_eq!(classes.alloc(8, &mut backing), Some(Slice(12, 20)));
        assert_eq!(classes.alloc(3, &mut backing), None::<Slice<u32>>);
        classes.dealloc(Slice(12, 20), &mut backing);
        assert_eq!(classes.alloc(3, &mut backing), Some(Slice(12, 16)));
        assert_eq!(classes.alloc(2, &mut backing), Some(Slice(16, 20)));

        classes.dealloc(Slice(0, 4), &mut backing);
        FreeSlices::<[_], u32>::clear(&mut classes, &mut backing);
        assert_eq!(classes.alloc(4, &mut backing), None::<Slice<u32>>);
    }

    fn check_exp2_size_classes<const N: usize, const B: usize>() {
        let classes = Exp2Size::<N, B>;
        assert_eq!(classes.size_class_contained(0), 0);
        assert_eq!(classes.size_class_containing(0), 0);
        assert_eq!(classes.round_up_capacity(0), 0);
        assert_eq!(classes.round_down_capacity(0), 0);
        let mut cap = 1usize << B;
        let mul = 1usize << N;
        for i in 1.. {
            assert_eq!(classes.capacity(i), cap);
            assert_eq!(classes.size_class_exact(cap), i);
            assert_eq!(classes.size_class_contained(cap), i);
            assert_eq!(classes.size_class_containing(cap), i);
            assert_eq!(classes.size_class_containing(cap + 1), i + 1);
            assert_eq!(classes.round_up_capacity(cap), cap);
            assert_eq!(classes.round_down_capacity(cap), cap);
            assert_eq!(
                classes.round_down_capacity(cap - 1),
                if cap == 1usize << B { 0 } else { cap / mul }
            );
            if cap > 2 {
                assert_eq!(classes.size_class_containing(cap - 1), i);
                assert_eq!(classes.round_down_capacity(cap + 1), cap);
                assert_eq!(classes.size_class_contained(cap - 1), i.saturating_sub(1))
            }
            if let Some(c) = cap.checked_mul(mul) {
                assert_eq!(classes.round_up_capacity(cap + 1), c);
                cap = c
            } else {
                break;
            }
        }
    }

    #[test]
    fn exp2_size_classes() {
        check_exp2_size_classes::<1, 0>();
        check_exp2_size_classes::<1, 1>();
        check_exp2_size_classes::<1, 2>();
        check_exp2_size_classes::<1, 3>();
        check_exp2_size_classes::<1, 4>();
        check_exp2_size_classes::<1, 5>();
        check_exp2_size_classes::<2, 0>();
        check_exp2_size_classes::<2, 1>();
        check_exp2_size_classes::<2, 2>();
        check_exp2_size_classes::<2, 3>();
        check_exp2_size_classes::<2, 4>();
        check_exp2_size_classes::<2, 5>();
        check_exp2_size_classes::<3, 0>();
        check_exp2_size_classes::<3, 1>();
        check_exp2_size_classes::<3, 2>();
        check_exp2_size_classes::<3, 3>();
        check_exp2_size_classes::<3, 4>();
        check_exp2_size_classes::<3, 5>();
        check_exp2_size_classes::<4, 0>();
        check_exp2_size_classes::<4, 1>();
        check_exp2_size_classes::<4, 2>();
        check_exp2_size_classes::<4, 3>();
        check_exp2_size_classes::<4, 4>();
        check_exp2_size_classes::<4, 5>();
    }
}
