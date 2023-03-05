/*!
A generational arena implementation
*/
use crate::slot::{InitFrom, Slot};

/// A slot supporting a generation counter
pub trait GenerationSlot<K>: Slot<K> {
    /// The type of this generation counter
    type GenerationIx: GenerationCounter;
    /// The current generation
    fn generation(&self) -> Self::GenerationIx;
    /// Increment the current generation
    fn inc_generation(&mut self);
}

/// A generation counter
pub trait GenerationCounter: Eq + Default {
    /// Increment the current generation
    fn inc_generation(&mut self);
}

impl GenerationCounter for () {
    #[inline(always)]
    fn inc_generation(&mut self) {}
}

macro_rules! wrapping_generation_counter {
    ($ty:ty) => {
        impl GenerationCounter for $ty {
            /// Increment the current generation
            fn inc_generation(&mut self) {
                *self = self.wrapping_add(1)
            }
        }
    };
}

wrapping_generation_counter!(u8);
wrapping_generation_counter!(u16);
wrapping_generation_counter!(u32);
wrapping_generation_counter!(u64);
wrapping_generation_counter!(u128);

/// Augment a slot or a key with a generation counter
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Generation<S, G>(pub S, pub G);

impl<S, G, V> InitFrom<V> for Generation<S, G>
where
    S: InitFrom<V>,
    G: GenerationCounter,
{
    #[inline(always)]
    fn from_value(value: V) -> Self
    where
        Self: Sized,
    {
        Generation(S::from_value(value), G::default())
    }

    #[inline(always)]
    fn set_value(&mut self, new: V) {
        self.0.set_value(new)
    }
}

impl<S, G, K> Slot<K> for Generation<S, G>
where
    S: Slot<K>,
    G: GenerationCounter,
{
    type Value = S::Value;

    #[inline(always)]
    fn into_value(self) -> Self::Value {
        self.0.into_value()
    }

    #[inline(always)]
    fn key(&self) -> K {
        self.0.key()
    }

    #[inline(always)]
    fn from_key(key: K) -> Self {
        Generation(S::from_key(key), G::default())
    }

    #[inline]
    fn set_key(&mut self, new: K) {
        self.1.inc_generation();
        self.0.set_key(new);
    }

    #[inline]
    fn swap_key(&mut self, new: K) -> Self::Value {
        self.1.inc_generation();
        self.0.swap_key(new)
    }

    #[inline(always)]
    fn swap_value(&mut self, new: Self::Value) -> Self::Value {
        self.0.swap_value(new)
    }
}

impl<S, G, K> GenerationSlot<K> for Generation<S, G>
where
    G: GenerationCounter + Clone,
    S: Slot<K>,
{
    type GenerationIx = G;

    #[inline(always)]
    fn generation(&self) -> Self::GenerationIx {
        self.1.clone()
    }

    #[inline(always)]
    fn inc_generation(&mut self) {
        self.1.inc_generation()
    }
}
