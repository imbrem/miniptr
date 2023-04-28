/*!
A free list implementation for a slice allocator
*/
use crate::{index::ContiguousIx, slot::KeySlot};

/// A set of free lists for a given capacity
pub trait FreeLists<B: ?Sized, K> {
    /// Allocate a slice in the backing, returning it's index
    ///
    /// Returns `None` on failure
    #[must_use]
    fn alloc(&mut self, capacity: usize, backing: &mut B) -> Option<Alloc<K>>;

    /// Deallocate a slice in the backing, putting it on the free list, potentially with a different capacity
    ///
    /// If the slot has not been previously alloc'ed or placed into the backing as a valid key, the behaviour is unspecified.
    fn dealloc(&mut self, alloc: Alloc<K>, backing: &mut B);
    
    /// Clear this free list, resetting it
    fn clear(&mut self, backing: &mut B);
}

/// A slice allocation
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Alloc<K> {
    /// The index of the free list
    pub index: K,
    /// The capacity of the free list
    pub capacity: usize,
}

/// An intrusive list for each size class
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct IntrusiveClasses<S> {
    free_heads: Vec<usize>,
    size_classes: S,
}

impl<K, S, T> FreeLists<[T], K> for IntrusiveClasses<S>
where
    K: ContiguousIx,
    S: SizeClasses,
    T: KeySlot<K>,
{
    #[inline]
    fn alloc(&mut self, capacity: usize, backing: &mut [T]) -> Option<Alloc<K>> {
        let size_class = self.size_classes.size_class_containing(capacity);
        let free_head = self.free_heads.get_mut(size_class as usize)?;
        let ix = free_head.index();
        let next_free = backing.get(ix)?.key();
        let next_index = next_free.index();
        let old_free_head = *free_head;
        *free_head = if next_index == old_free_head {
            usize::MAX
        } else {
            next_index
        };
        Some(Alloc {
            index: K::new_unchecked(old_free_head),
            capacity: capacity,
        })
    }

    #[inline]
    fn dealloc(&mut self, alloc: Alloc<K>, backing: &mut [T]) {
        let size_class = self.size_classes.size_class_contained(alloc.capacity);
        self.free_heads.resize(size_class as usize + 1, usize::MAX);
        let old_free_head = self.free_heads[size_class as usize];
        let new_free_head = alloc.index.index();
        backing[new_free_head].set_key(K::try_new(old_free_head).unwrap_or(alloc.index));
        self.free_heads[size_class as usize] = new_free_head;
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
    /// Returns `u32::MAX` if there is no matching size class
    fn size_class_containing(&self, capacity: usize) -> u32;
    /// Get the index of the largest size class contained within this capacity
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
}

/// Constant exponential size classes where `ExpSize<N, B>.capacity(n) = 2**(n*N + B)`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
        if capacity == 0 {
            return 0;
        }
        capacity.ilog2().saturating_sub(B as u32) / N as u32
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity(&self, size_class: u32) -> usize {
        //TODO: optimize
        1usize << (B as u32 + size_class * N as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_exp2_size_classes<const N: usize, const B: usize>() {
        let classes = Exp2Size::<N, B>;
        let mut cap = 1usize << B;
        let mul = 1usize << N;
        for i in 0.. {
            assert_eq!(classes.capacity(i), cap);
            assert_eq!(classes.size_class_contained(cap), i);
            assert_eq!(classes.size_class_containing(cap), i);
            assert_eq!(classes.size_class_containing(cap + 1), i + 1);
            if cap > 2 {
                assert_eq!(classes.size_class_containing(cap - 1), i);
                assert_eq!(classes.size_class_contained(cap - 1), i.saturating_sub(1))
            }
            if let Some(c) = cap.checked_mul(mul) {
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
