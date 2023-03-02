/*!
Traits for index types
*/

/// A type which can contain a contiguous integer index between `0` and `n`
///
/// The implementations of `Eq`, `Ord`, and `PartialOrd` should be consistent with that on `n` for values constructed via `Self::new(n)`.
use bytemuck::TransparentWrapper;
pub trait ContiguousIx: Copy + Eq + Ord + PartialOrd {
    /// The maximum index this type can hold
    const MAX_INDEX: usize;

    /// Create a new index from an integer, panicking if `ix` is out of bounds
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new(ix: usize) -> Self {
        Self::try_new(ix).expect("index not representable")
    }

    /// Create a new index from an integer, returning an unspecified value or panicking if `ix` is out of bounds
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn new_unchecked(ix: usize) -> Self {
        Self::new(ix)
    }

    /// Create a new index from an integer, returning `None` if `ix` is out of bounds
    fn try_new(ix: usize) -> Option<Self>;

    /// Get the index value represented by this `ContiguousIx`.
    ///
    /// For a valid index `i` created by `Self::try_new(n)`, `i.index()` should
    /// return `n`. The returned value should always be less than or equal to
    /// `Self::MAX_INDEX`.
    fn index(self) -> usize;

    /// Whether this index is zero
    ///
    /// Should be equivalent to `self == Self::new(0)`
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn is_zero(self) -> bool {
        self == Self::new(0)
    }
}

macro_rules! primitive_contiguous_ix {
    ($ty:ty) => {
        impl ContiguousIx for $ty {
            const MAX_INDEX: usize = <$ty>::MAX as usize;

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_new(ix: usize) -> Option<Self> {
                if ix > <$ty>::MAX as usize {
                    None
                } else {
                    Some(ix as $ty)
                }
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn new_unchecked(ix: usize) -> Self {
                ix as $ty
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn index(self) -> usize {
                self as usize
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn new(ix: usize) -> Self {
                if ix > <$ty>::MAX as usize {
                    panic!("{ix} is not representable as a {}", stringify!($ty))
                } else {
                    ix as $ty
                }
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn is_zero(self) -> bool {
                self == 0
            }
        }
    };
}

primitive_contiguous_ix!(u8);
primitive_contiguous_ix!(u16);
primitive_contiguous_ix!(u32);
primitive_contiguous_ix!(u64);
primitive_contiguous_ix!(u128);
primitive_contiguous_ix!(usize);
primitive_contiguous_ix!(i8);
primitive_contiguous_ix!(i16);
primitive_contiguous_ix!(i32);
primitive_contiguous_ix!(i64);
primitive_contiguous_ix!(i128);
primitive_contiguous_ix!(isize);

/// A wrapper around a primitive integer type for which [`ContiguousIx`] maps `0..n` to `-1..-(n + 1)`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, TransparentWrapper)]
#[repr(transparent)]
pub struct Neg<T>(pub T);

impl<T: PartialOrd> PartialOrd for Neg<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.partial_cmp(&other.0)?.reverse())
    }
}

impl<T: Ord> Ord for Neg<T> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

macro_rules! negative_contiguous_ix {
    ($ty:ty) => {
        impl ContiguousIx for Neg<$ty> {
            const MAX_INDEX: usize = <$ty>::MAX as usize;

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn try_new(ix: usize) -> Option<Self> {
                if ix > Self::MAX_INDEX as usize {
                    None
                } else {
                    Some(Neg((-(ix as isize) - 1) as $ty))
                }
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn new_unchecked(ix: usize) -> Self {
                Neg((-(ix as isize) - 1) as $ty)
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn index(self) -> usize {
                (-(self.0 as isize + 1)) as usize
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn new(ix: usize) -> Self {
                if ix > Self::MAX_INDEX as usize {
                    panic!("{ix} is not representable as a {}", stringify!($ty))
                } else {
                    Neg((-(ix as isize) - 1) as $ty)
                }
            }

            #[cfg_attr(not(tarpaulin), inline(always))]
            fn is_zero(self) -> bool {
                self.0 == -1
            }
        }
    };
}

negative_contiguous_ix!(i8);
negative_contiguous_ix!(i16);
negative_contiguous_ix!(i32);
negative_contiguous_ix!(i64);
negative_contiguous_ix!(i128);
negative_contiguous_ix!(isize);

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn u8_contiguous_ix() {
        assert_eq!(u8::MAX_INDEX, u8::MAX as usize);
        for i in 0..=u8::MAX_INDEX {
            assert_eq!(u8::new(i), i as u8);
            assert_eq!(u8::try_new(i), Some(i as u8));
            assert_eq!(u8::new_unchecked(i), i as u8);
            assert_eq!(u8::new(i).index(), i);
            assert_eq!(u8::new(i).is_zero(), i == 0);
        }
        assert_eq!(u8::try_new(256), None);
        assert_eq!(u8::new_unchecked(256), 0);
    }

    #[test]
    fn i8_continuous_ix() {
        assert_eq!(i8::MAX_INDEX, i8::MAX as usize);
        for i in 0..=i8::MAX_INDEX {
            assert_eq!(i8::new(i), i as i8);
            assert_eq!(i8::try_new(i), Some(i as i8));
            assert_eq!(i8::new_unchecked(i), i as i8);
            assert_eq!(i8::new(i).index(), i);
            assert_eq!(i8::new(i).is_zero(), i == 0);
        }
        assert_eq!(i8::try_new(128), None);
        assert_eq!(i8::new_unchecked(128), -128);

        assert_eq!(Neg::<i8>::MAX_INDEX, i8::MAX as usize);
        for i in 0..=Neg::<i8>::MAX_INDEX {
            let ni = -(i as i8) - 1;
            assert_eq!(Neg::<i8>::new(i), Neg(ni));
            assert_eq!(Neg::<i8>::try_new(i), Some(Neg(ni)));
            assert_eq!(Neg::<i8>::new_unchecked(i), Neg(ni));
            assert_eq!(Neg::<i8>::new(i).index(), i);
            assert_eq!(Neg::<i8>::new(i).is_zero(), i == 0);
        }
        assert_eq!(Neg::<i8>::try_new(128), None);
        assert_eq!(Neg::<i8>::new_unchecked(128), Neg(127));

        assert_eq!(Neg(5).cmp(&Neg(3)), Ordering::Less);
        assert_eq!(Neg(5).partial_cmp(&Neg(3)), Some(Ordering::Less))
    }

    #[test]
    #[should_panic]
    fn u8_contiguous_ix_overflow() {
        u8::new(256);
    }

    #[test]
    #[should_panic]
    fn i8_negative_ix_overflow() {
        Neg::<i8>::new(128);
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    struct Mix(u32);

    impl ContiguousIx for Mix {
        const MAX_INDEX: usize = 10;

        fn try_new(ix: usize) -> Option<Self> {
            if ix > 10 {
                None
            } else {
                Some(Mix(ix as u32))
            }
        }

        fn index(self) -> usize {
            self.0 as usize
        }
    }

    #[test]
    fn default_contiguous_ix_impl() {
        for i in 0..=10 {
            assert_eq!(Mix::new(i), Mix(i as u32));
            assert_eq!(Mix::new_unchecked(i), Mix(i as u32));
            assert_eq!(Mix::try_new(i), Some(Mix(i as u32)));
            assert_eq!(Mix::new(i).is_zero(), i == 0);
        }
        assert_eq!(Mix::try_new(11), None);
    }
}
