/*!
A pool of slices
*/

// /// A pool of slices
// pub struct SlicePool<T, F> {
//     /// The backing memory of this slice pool
//     backing: Vec<T>,
//     /// The free lists of this slice pool
//     free: F,
// }

/// A set of free lists
pub trait FreeLists<T> {
    /// Get a free list with the given capacity
    ///
    /// Return `None` if no free lists with the given capacity are available
    fn alloc(&mut self, capacity: usize, backing: &mut [T]) -> Option<FreeList>;
    /// Free a list with the given capacity
    ///
    /// Return an error on failure
    fn dealloc(&mut self, index: usize, capacity: usize, backing: &mut [T]) -> Result<(), ()>;
}

/// A free list
pub struct FreeList {
    /// The index of the free list
    pub index: usize,
    /// The capacity of the free list
    pub capacity: usize,
}

/// A mapping from capacities to size classes
pub trait SizeClasses {
    /// Get the index of the smallest size class containing this capacity
    ///
    /// Returns `usize::MAX` if there is no matching size class
    fn size_class_containing(&self, capacity: usize) -> usize;
    /// Get the index of the largest size class contained within this capacity
    fn size_class_contained(&self, capacity: usize) -> usize;
    /// Get the capacity of a given size class
    ///
    /// Returns an unspecified value if `size_class` is out of bounds
    fn capacity(&self, size_class: usize) -> usize;
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
    fn size_class_containing(&self, capacity: usize) -> usize {
        1 + ((usize::BITS - capacity.leading_zeros()) as usize).saturating_sub(B + 1) / N
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn size_class_contained(&self, capacity: usize) -> usize {
        ((usize::BITS - capacity.leading_zeros()) as usize).saturating_sub(B) / N
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn capacity(&self, size_class: usize) -> usize {
        1usize << (B + size_class * N)
    }
}

#[cfg(test)]
mod test {
    use crate::pool::slice::SizeClasses;

    use super::Exp2Size;

    fn check_exp2_size_classes<const N: usize, const B: usize>() {
        let classes = Exp2Size::<N, B>;
        let mut cap = 1usize << B;
        let mul = 1usize << N;
        for i in 0.. {
            assert_eq!(classes.capacity(i), cap);
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
