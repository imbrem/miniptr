/*!
A pool of slices
*/
pub mod free;

/// A pool of slices
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SlicePool<T, F> {
    /// The backing memory of this slice pool
    backing: Vec<T>,
    /// The free lists of this slice pool
    free: F,
}

impl<T, F> SlicePool<T, F> {

}