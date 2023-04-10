/*!
Traits for containers implementing arrays
*/

use super::{
    map::{GetIndex, GetIndexMut},
    stack::StackPool,
    *,
};

/// A [`Pool`] allocating immutable arrays containing elements of type `Self::Elem`
///
/// Automatically implemented for any [`ContainerPool`] implementing [`LenPool`] and [`GetIndex<K, usize, Self::Elem>`]
pub trait ArrayRefPool<K>: ContainerPool<K> + LenPool<K> + GetIndex<K, usize, Self::Elem> {}
impl<P, K> ArrayRefPool<K> for P where
    P: ContainerPool<K> + LenPool<K> + GetIndex<K, usize, Self::Elem>
{
}

/// A [`Pool`] allocating mutable arrays containing elements of type `Self::Elem`
///
/// Automatically implemented for any [`ContainerPool`] implementing [`LenPool`] and [`GetIndexMut<K, usize, Self::Elem>`]
pub trait ArrayMutPool<K>:
    ContainerPool<K> + LenPool<K> + GetIndexMut<K, usize, Self::Elem>
{
}
impl<P, K> ArrayMutPool<K> for P where
    P: ContainerPool<K> + LenPool<K> + GetIndexMut<K, usize, Self::Elem>
{
}

/// A [`Pool`] allocating arrays containing elements of type `Self::Elem`
///
/// Automatically implemented for any [`Pool`] implementing [`ArrayRefPool`] and [`ArrayMutPool`]
pub trait ArrayPool<K>: ArrayRefPool<K> + ArrayMutPool<K> {}
impl<P, K> ArrayPool<K> for P where P: ArrayRefPool<K> + ArrayMutPool<K> {}

/// A [`Pool`] allocating immutable slices of `Self::Elem`
pub trait SliceRefPool<K>: ArrayRefPool<K> + GetRef<K, [Self::Elem]> {}

/// A [`Pool`] allocating mutable slices of `Self::Elem`
pub trait SliceMutPool<K>: ArrayMutPool<K> + GetMut<K, [Self::Elem]> {}

/// A [`Pool`] allocating slices of `Self::Item`
///
/// Automatically implemented for any [`Pool`] implementing [`SliceRefPool`] and [`SliceMutPool`]
pub trait SlicePool<K>: SliceRefPool<K> + SliceMutPool<K> {}
impl<P, K> SlicePool<K> for P where P: SliceRefPool<K> + SliceMutPool<K> {}

/// A pool containing `ArrayList`s: growable arrays
///
/// Automatically implemented for any [`Pool`] implementing [`ArrayPool`] and [`StackPool`].
pub trait ArrayListPool<K>: ArrayPool<K> + StackPool<K> {}
impl<P, K> ArrayListPool<K> for P where P: ArrayPool<K> + StackPool<K> {}

/// A pool containing growable arrays allocated in a contiguous slice
///
/// Automatically implemented for any [`Pool`] implementing [`SlicePool`] and [`StackPool`].
pub trait VecPool<K>: SlicePool<K> + StackPool<K> {}
impl<P, K> VecPool<K> for P where P: SlicePool<K> + StackPool<K> {}
