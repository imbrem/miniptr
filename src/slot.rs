/*!
Traits for slots in arena-based data structures
*/
use bytemuck::TransparentWrapper;
use either::Either;

/// A type which can be initialized given a value of type `V`, potentially re-using existing resources
pub trait InitFrom<V> {
    /// Create a slot from a value
    fn from_value(value: V) -> Self
    where
        Self: Sized;

    /// Set this slot to a value
    fn set_value(&mut self, new: V);
}

/// A type which can contain either
/// - A value of type `Self::Value`
/// - A key of type `K`
pub trait Slot<K>: Sized + InitFrom<Self::Value> {
    /// The type of values which may be stored in this slot
    type Value;

    /// If this slot contains a value, return it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value if this slot does not contain a value
    fn into_value(self) -> Self::Value;

    /// If this slot contains a key, return it
    ///
    /// A slot is guaranteed to contain a key if created using `Self::from_key`
    ///
    /// Panic or return an arbitrary value otherwise
    fn key(&self) -> K;

    /// Create a slot from a key
    fn from_key(key: K) -> Self;

    /// Create a slot from either a value or key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_either(value: Either<K, Self::Value>) -> Self {
        value.either(Self::from_key, Self::from_value)
    }

    /// Set this slot to either a value or a key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_slot(&mut self, new: Either<K, Self::Value>) {
        *self = Self::from_either(new);
    }

    /// Set this slot to a key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_key(&mut self, new: K) {
        *self = Self::from_key(new)
    }

    /// Take this slot's value, replacing it with another
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap(&mut self, new: Either<K, Self::Value>) -> Self::Value {
        let mut result = Self::from_either(new);
        std::mem::swap(&mut result, self);
        result.into_value()
    }

    /// Take this slot's value, replacing it with a key
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap_key(&mut self, new: K) -> Self::Value {
        let mut result = Self::from_key(new);
        std::mem::swap(&mut result, self);
        result.into_value()
    }

    /// Take this slot's value, replacing it with a new one
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap_value(&mut self, new: Self::Value) -> Self::Value {
        let mut result = Self::from_value(new);
        std::mem::swap(&mut result, self);
        result.into_value()
    }
}

/// A slot which can be dereferenced
pub trait SlotRef<K>: Slot<K> {
    /// If this slot contains a value, return a reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value otherwise
    fn value(&self) -> &Self::Value;
}

/// A slot which can be mutably dereferenced
pub trait SlotMut<K>: Slot<K> {
    /// If this slot contains a value, return a mutable reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value otherwise
    fn value_mut(&mut self) -> &mut Self::Value;
}

/// The identity slot: contains a key, which can be interpreted as either a key or a value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, TransparentWrapper)]
#[repr(transparent)]
pub struct IdSlot<K>(K);

impl<K> InitFrom<K> for IdSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_value(value: K) -> Self {
        IdSlot(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_value(&mut self, new: K) {
        self.0 = new
    }
}

impl<K> Slot<K> for IdSlot<K>
where
    K: Clone,
{
    type Value = K;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn into_value(self) -> Self::Value {
        self.0
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key(&self) -> K {
        self.0.clone()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_key(key: K) -> Self {
        IdSlot(key)
    }
}

impl<K> SlotRef<K> for IdSlot<K>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value(&self) -> &Self::Value {
        &self.0
    }
}

impl<K> SlotMut<K> for IdSlot<K>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.0
    }
}

/// A slot which can be queried to see whether it contains a key or a value
///
/// Note a `CheckedSlot` is always guaranteed to contain at least one of a key or value (unlike a [`Slot`]!)
pub trait CheckedSlot<K>: Slot<K> {
    /// Whether this slot contains a value
    fn has_value(&self) -> bool;

    /// Whether this slot contains a key
    fn has_key(&self) -> bool;

    /// If this slot contains a value, return it; otherwise, return `None`
    fn try_into_value(self) -> Option<Self::Value> {
        if self.has_value() {
            Some(self.into_value())
        } else {
            None
        }
    }

    /// If this slot contains a value, return a reference to it
    fn try_value(&self) -> Option<&Self::Value>
    where
        Self: SlotRef<K>,
    {
        if self.has_value() {
            Some(self.value())
        } else {
            None
        }
    }

    /// If this slot contains a value, return a mutable reference to it; otherwise, return `None`
    fn try_value_mut(&mut self) -> Option<&mut Self::Value>
    where
        Self: SlotMut<K>,
    {
        if self.has_value() {
            Some(self.value_mut())
        } else {
            None
        }
    }

    /// If this slot contains a key, return it; otherwise, return `None`
    ///
    /// A slot is guaranteed to contain a key if created using `Self::from_key`
    fn try_key(&self) -> Option<K> {
        if self.has_key() {
            Some(self.key())
        } else {
            None
        }
    }

    /// Convert this slot into either a key or a value
    ///
    /// If this slot can be interpreted as both a key and value, prefer the key
    fn into_either(self) -> Either<K, Self::Value> {
        if self.has_key() {
            Either::Left(self.key())
        } else {
            Either::Right(self.into_value())
        }
    }

    /// Get this slot into as a key or a reference to a value
    ///
    /// If this slot can be interpreted as both a key and value, prefer the key
    fn as_either(&self) -> Either<K, &Self::Value>
    where
        Self: SlotRef<K>,
    {
        if self.has_key() {
            Either::Left(self.key())
        } else {
            Either::Right(self.value())
        }
    }

    /// Get this slot into as a key or a mutable reference to a value
    ///
    /// If this slot can be interpreted as both a key and value, prefer the key
    fn as_either_mut(&mut self) -> Either<K, &mut Self::Value>
    where
        Self: SlotMut<K>,
    {
        if self.has_key() {
            Either::Left(self.key())
        } else {
            Either::Right(self.value_mut())
        }
    }
}

impl<K, V> InitFrom<V> for Either<K, V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_value(value: V) -> Self {
        Self::Right(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_value(&mut self, new: V) {
        *self = Self::Right(new)
    }
}

impl<K, V> Slot<K> for Either<K, V>
where
    K: Clone,
{
    type Value = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_key(key: K) -> Self {
        Self::Left(key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn into_value(self) -> Self::Value {
        self.right().unwrap()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key(&self) -> K {
        self.as_ref().left().unwrap().clone()
    }
}

impl<K, V> SlotRef<K> for Either<K, V>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value(&self) -> &Self::Value {
        self.as_ref().right().unwrap()
    }
}

impl<K, V> SlotMut<K> for Either<K, V>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value_mut(&mut self) -> &mut Self::Value {
        self.as_mut().right().unwrap()
    }
}

impl<K, V> CheckedSlot<K> for Either<K, V>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn has_value(&self) -> bool {
        self.is_right()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn has_key(&self) -> bool {
        self.is_left()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_value(self) -> Option<Self::Value> {
        self.right()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value(&self) -> Option<&Self::Value> {
        self.as_ref().right()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value_mut(&mut self) -> Option<&mut Self::Value> {
        self.as_mut().right()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_key(&self) -> Option<K> {
        self.as_ref().left().cloned()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn into_either(self) -> Either<K, Self::Value> {
        self
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn as_either(&self) -> Either<K, &Self::Value> {
        self.as_ref().map_left(Clone::clone)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn as_either_mut(&mut self) -> Either<K, &mut Self::Value> {
        self.as_mut().map_left(|x| x.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn either_slot_impl() {
        let mut e: Either<u8, u16> = Either::Left(5);
        assert_eq!(e.key(), 5);
        assert_eq!(e.try_key(), Some(5));
        assert_eq!(e.try_value(), None);
        assert_eq!(e.try_value_mut(), None);
        assert_eq!(e.try_into_value(), None);
        assert!(!e.has_value());
        assert!(e.has_key());
        assert_eq!(e.into_either(), e);
        assert_eq!(e.as_either().left(), e.left());
        assert_eq!(e.as_either_mut().left(), e.left());
        e.set_key(7);
        assert_eq!(e.key(), 7);
        assert_eq!(e.try_key(), Some(7));
        assert_eq!(e.try_value(), None);
        assert!(!e.has_value());
        assert!(e.has_key());
        assert_eq!(e.into_either(), e);
        assert_eq!(e.as_either().left(), e.left());
        assert_eq!(e.as_either_mut().left(), e.left());
        e.set_value(55);
        assert!(e.has_value());
        assert!(!e.has_key());
        assert_eq!(e.into_value(), 55);
        assert_eq!(e.try_value(), Some(&55));
        assert_eq!(e.try_key(), None);
        assert_eq!(e.try_value_mut(), Some(&mut 55));
        assert_eq!(e.try_into_value(), Some(55));
        assert_eq!(*e.value(), 55);
        assert_eq!(*e.value_mut(), 55);
        assert_eq!(e.swap_key(9), 55);
        assert_eq!(e.try_key(), Some(9));
        e.set_slot(Either::Right(98));
        assert_eq!(e.swap(Either::Right(99)), 98);
        assert_eq!(e.swap_value(32), 99);
        assert_eq!(*e.value(), 32);
        assert_eq!(*e.value_mut(), 32);
        *e.value_mut() = 15;
        assert_eq!(*e.value(), 15);
        assert_eq!(*e.value_mut(), 15);
        assert_eq!(e.as_either(), Either::Right(&15));
        assert_eq!(e.as_either_mut(), Either::Right(&mut 15));
    }

    #[test]
    fn id_slot_impl() {
        assert_eq!(IdSlot(5).value(), &5);
        assert_eq!(IdSlot(6).value_mut(), &mut 6);
    }

    #[derive(PartialEq, Copy, Clone)]
    enum MySlot {
        Key(u8),
        Value(u16),
    }

    impl InitFrom<u16> for MySlot {
        fn from_value(value: u16) -> Self {
            MySlot::Value(value)
        }

        fn set_value(&mut self, new: u16) {
            *self = MySlot::Value(new)
        }
    }

    impl Slot<u8> for MySlot {
        type Value = u16;

        fn into_value(self) -> u16 {
            match self {
                MySlot::Key(_) => unreachable!(),
                MySlot::Value(v) => v,
            }
        }

        fn key(&self) -> u8 {
            match self {
                MySlot::Key(k) => *k,
                MySlot::Value(_) => unreachable!(),
            }
        }

        fn from_key(key: u8) -> Self {
            MySlot::Key(key)
        }
    }

    impl SlotRef<u8> for MySlot {
        fn value(&self) -> &u16 {
            match self {
                MySlot::Key(_) => unreachable!(),
                MySlot::Value(v) => v,
            }
        }
    }

    impl SlotMut<u8> for MySlot {
        fn value_mut(&mut self) -> &mut u16 {
            match self {
                MySlot::Key(_) => unreachable!(),
                MySlot::Value(v) => v,
            }
        }
    }

    impl CheckedSlot<u8> for MySlot {
        fn has_value(&self) -> bool {
            matches!(self, MySlot::Value(_))
        }

        fn has_key(&self) -> bool {
            matches!(self, MySlot::Key(_))
        }
    }

    #[test]
    fn default_slot_impl() {
        let mut e = MySlot::Key(5);
        assert_eq!(e.key(), 5);
        assert_eq!(e.try_key(), Some(5));
        assert_eq!(e.try_value(), None);
        assert_eq!(e.try_value_mut(), None);
        assert_eq!(e.try_into_value(), None);
        assert!(!e.has_value());
        assert!(e.has_key());
        assert_eq!(e.into_either(), Either::Left(5));
        assert_eq!(e.as_either().left(), Some(5));
        assert_eq!(e.as_either_mut().left(), Some(5));
        e.set_key(7);
        assert_eq!(e.key(), 7);
        assert_eq!(e.try_key(), Some(7));
        assert_eq!(e.try_value(), None);
        assert!(!e.has_value());
        assert!(e.has_key());
        assert_eq!(e.into_either(), Either::Left(7));
        assert_eq!(e.as_either().left(), Some(7));
        assert_eq!(e.as_either_mut().left(), Some(7));
        e.set_value(55);
        assert!(e.has_value());
        assert!(!e.has_key());
        assert_eq!(e.into_value(), 55);
        assert_eq!(e.try_value(), Some(&55));
        assert_eq!(e.try_key(), None);
        assert_eq!(e.try_value_mut(), Some(&mut 55));
        assert_eq!(e.try_into_value(), Some(55));
        assert_eq!(*e.value(), 55);
        assert_eq!(*e.value_mut(), 55);
        assert_eq!(e.swap_key(9), 55);
        assert_eq!(e.try_key(), Some(9));
        e.set_slot(Either::Right(98));
        assert_eq!(e.swap(Either::Right(99)), 98);
        assert_eq!(e.swap_value(32), 99);
        assert_eq!(*e.value(), 32);
        assert_eq!(*e.value_mut(), 32);
        *e.value_mut() = 15;
        assert_eq!(*e.value(), 15);
        assert_eq!(*e.value_mut(), 15);
        assert_eq!(e.into_either(), Either::Right(15));
        assert_eq!(e.as_either(), Either::Right(&15));
        assert_eq!(e.as_either_mut(), Either::Right(&mut 15));
    }
}
