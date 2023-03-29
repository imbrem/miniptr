/*!
Traits for slots in arena-based data structures
*/
use bytemuck::TransparentWrapper;
use either::Either;

/// A type which can be initialized given a value of type `V`, potentially re-using existing resources
pub trait InitFrom<V> {
    /// Create a slot from a value
    #[must_use]
    fn from_value(value: V) -> Self
    where
        Self: Sized;

    /// Set this slot to a value
    fn set_value(&mut self, new: V);
}

/// A type which can contain a single value of type `Self::Value`
pub trait Slot: Sized + InitFrom<Self::Value> {
    /// The type of values which may be stored in this slot
    type Value;

    /// If this slot contains a value, return it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Returns an arbitrary value if this slot does not contain a value
    #[must_use]
    fn try_into_value(self) -> Option<Self::Value>;

    /// If this slot contains a value, return it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn into_value(self) -> Self::Value {
        self.try_into_value().expect("slot does not contain value")
    }

    /// Take this slot's value, replacing it with a new one
    ///
    /// Return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn try_swap_value(&mut self, new: Self::Value) -> Option<Self::Value> {
        let mut result = Self::from_value(new);
        std::mem::swap(&mut result, self);
        result.try_into_value()
    }

    /// Take this slot's value, replacing it with a new one
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn swap_value(&mut self, new: Self::Value) -> Self::Value {
        self.try_swap_value(new)
            .expect("slot does not contain value")
    }

    /// Create a new slot with the default value
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn default_value() -> Self
    where
        Self: Sized,
        Self::Value: Default,
    {
        Self::from_value(Self::Value::default())
    }

    /// Set this slot to the default value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_default_value(&mut self)
    where
        Self::Value: Default,
    {
        self.set_value(Self::Value::default())
    }

    /// Take this slot's value, replacing it with the default
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap_default_value(&mut self) -> Self::Value
    where
        Self::Value: Default,
    {
        self.swap_value(Self::Value::default())
    }
}

/// A type which can contain a single value of type `Self::Value`, which can be extracted
pub trait RemoveSlot: Slot {
    /// Take this slot's value, replacing it with a new one
    ///
    /// Return an arbitrary value if this slot does not contain a value
    #[must_use]
    fn try_remove_value(&mut self) -> Option<Self::Value>;

    /// Take this slot's value, replacing it with a new one
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    #[must_use]
    fn remove_value(&mut self) -> Self::Value {
        self.try_remove_value()
            .expect("slot does not contain value")
    }

    /// Delete this slot's value
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete_value(&mut self) {
        let _ = self.remove_value();
    }
}

/// A type which can contain either
/// - A value of type `Self::Value`
/// - A key of type `K`
pub trait KeySlot<K>: Slot {
    /// If this slot contains a key, return it
    ///
    /// A slot is guaranteed to contain a key if created using `Self::from_key`
    ///
    /// Returns an arbitrary value if this slot does not contain a key
    fn try_key(&self) -> Option<K>;

    /// If this slot contains a key, return it
    ///
    /// A slot is guaranteed to contain a key if created using `Self::from_key`
    ///
    /// Panics or returns an arbitrary value if this slot does not contain a key
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key(&self) -> K {
        self.try_key().expect("slot does not contain key")
    }

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
    /// Return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_swap(&mut self, new: Either<K, Self::Value>) -> Option<Self::Value> {
        match new {
            Either::Left(left) => self.try_swap_key(left),
            Either::Right(right) => self.try_swap_value(right),
        }
    }

    /// Take this slot's value, replacing it with a key
    ///
    /// Return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_swap_key(&mut self, new: K) -> Option<Self::Value> {
        let mut result = Self::from_key(new);
        std::mem::swap(&mut result, self);
        result.try_into_value()
    }

    /// Take this slot's value, replacing it with another
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap(&mut self, new: Either<K, Self::Value>) -> Self::Value {
        self.try_swap(new).expect("slot does not contain value")
    }

    /// Take this slot's value, replacing it with a key
    ///
    /// Panic or return an arbitrary value if this slot does not contain a value
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn swap_key(&mut self, new: K) -> Self::Value {
        self.try_swap_key(new).expect("slot does not contain value")
    }
}

/// A slot which can be dereferenced
pub trait SlotRef: Slot {
    /// If this slot contains a value, return a reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Returns an arbitrary value otherwise
    fn try_value(&self) -> Option<&Self::Value>;

    /// If this slot contains a value, return a reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value otherwise
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value(&self) -> &Self::Value {
        self.try_value().expect("slot does not contain value")
    }
}

/// A slot which can be mutably dereferenced
pub trait SlotMut: Slot {
    /// If this slot contains a value, return a mutable reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Returns an arbitrary value otherwise
    fn try_value_mut(&mut self) -> Option<&mut Self::Value>;

    /// If this slot contains a value, return a mutable reference to it
    ///
    /// A slot is guaranteed to contain a value if created using `Self::from_value`
    ///
    /// Panics or returns an arbitrary value otherwise
    fn value_mut(&mut self) -> &mut Self::Value {
        self.try_value_mut().expect("slot does not contain value")
    }
}

/// The identity slot: contains a key, which can be interpreted as either a key or a value
///
/// Values are removed by cloning the current value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, TransparentWrapper)]
#[repr(transparent)]
pub struct CloneSlot<K>(pub K);

impl<K> InitFrom<K> for CloneSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_value(value: K) -> Self {
        CloneSlot(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_value(&mut self, new: K) {
        self.0 = new
    }
}

impl<K> Slot for CloneSlot<K> {
    type Value = K;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_value(self) -> Option<Self::Value> {
        Some(self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn into_value(self) -> Self::Value {
        self.0
    }
}

impl<K> RemoveSlot for CloneSlot<K>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove_value(&mut self) -> Option<Self::Value> {
        Some(self.0.clone())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn remove_value(&mut self) -> Self::Value {
        self.0.clone()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn delete_value(&mut self) {}
}

impl<K> KeySlot<K> for CloneSlot<K>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_key(&self) -> Option<K> {
        Some(self.0.clone())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key(&self) -> K {
        self.0.clone()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_key(key: K) -> Self {
        CloneSlot(key)
    }
}

impl<K> SlotRef for CloneSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value(&self) -> Option<&Self::Value> {
        Some(&self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value(&self) -> &Self::Value {
        &self.0
    }
}

impl<K> SlotMut for CloneSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value_mut(&mut self) -> Option<&mut Self::Value> {
        Some(&mut self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.0
    }
}

/// The identity slot: contains a key, which can be interpreted as either a key or a value
///
/// Values are removed by replacing them with the default value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, TransparentWrapper)]
#[repr(transparent)]
pub struct DefaultSlot<K>(pub K);

impl<K> InitFrom<K> for DefaultSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_value(value: K) -> Self {
        DefaultSlot(value)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn set_value(&mut self, new: K) {
        self.0 = new
    }
}

impl<K> Slot for DefaultSlot<K> {
    type Value = K;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_value(self) -> Option<Self::Value> {
        Some(self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn into_value(self) -> Self::Value {
        self.0
    }
}

impl<K> RemoveSlot for DefaultSlot<K>
where
    K: Default,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_remove_value(&mut self) -> Option<Self::Value> {
        Some(self.swap_value(K::default()))
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn remove_value(&mut self) -> Self::Value {
        self.swap_value(K::default())
    }
}

impl<K> KeySlot<K> for DefaultSlot<K>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_key(&self) -> Option<K> {
        Some(self.0.clone())
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn key(&self) -> K {
        self.0.clone()
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_key(key: K) -> Self {
        DefaultSlot(key)
    }
}

impl<K> SlotRef for DefaultSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value(&self) -> Option<&Self::Value> {
        Some(&self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value(&self) -> &Self::Value {
        &self.0
    }
}

impl<K> SlotMut for DefaultSlot<K> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value_mut(&mut self) -> Option<&mut Self::Value> {
        Some(&mut self.0)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.0
    }
}

/// A slot which can be queried to see whether it contains a key or a value
///
/// Note a `CheckedSlot` is always guaranteed to contain at least one of a key or value (unlike a [`Slot`]!)
pub trait CheckedSlot<K>: KeySlot<K> {
    /// Whether this slot contains a value
    fn has_value(&self) -> bool;

    /// Whether this slot contains a key
    fn has_key(&self) -> bool;

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
        Self: SlotRef,
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
        Self: SlotMut,
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

impl<K, V> Slot for Either<K, V> {
    type Value = V;

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_into_value(self) -> Option<Self::Value> {
        self.right()
    }
}

impl<K, V> KeySlot<K> for Either<K, V>
where
    K: Clone,
{
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn from_key(key: K) -> Self {
        Self::Left(key)
    }

    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_key(&self) -> Option<K> {
        self.as_ref().left().cloned()
    }
}

impl<K, V> SlotRef for Either<K, V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value(&self) -> Option<&Self::Value> {
        self.as_ref().right()
    }
}

impl<K, V> SlotMut for Either<K, V> {
    #[cfg_attr(not(tarpaulin), inline(always))]
    fn try_value_mut(&mut self) -> Option<&mut Self::Value> {
        self.as_mut().right()
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
    fn default_slot_impl() {
        assert_eq!(DefaultSlot(5).try_key(), Some(5));
        assert_eq!(DefaultSlot(5).key(), 5);
        assert_eq!(DefaultSlot(5).try_into_value(), Some(5));
        assert_eq!(DefaultSlot(5).into_value(), 5);
        assert_eq!(DefaultSlot(5).try_value(), Some(&5));
        assert_eq!(DefaultSlot(6).try_value_mut(), Some(&mut 6));
        assert_eq!(DefaultSlot(5).value(), &5);
        assert_eq!(DefaultSlot(6).value_mut(), &mut 6);
        assert_eq!(DefaultSlot(5).try_swap(Either::Left(7)), Some(5));
        let mut slot = DefaultSlot(7);
        assert_eq!(slot.try_remove_value(), Some(7));
        assert_eq!(slot, DefaultSlot(0));
        assert_eq!(slot.remove_value(), 0);
        assert_eq!(slot, DefaultSlot(0));
        slot.set_value(5);
        assert_eq!(slot, DefaultSlot(5));
        slot.delete_value();
        assert_eq!(slot, DefaultSlot(0));
    }

    #[test]
    fn clone_slot_impl() {
        assert_eq!(CloneSlot(5).try_key(), Some(5));
        assert_eq!(CloneSlot(5).key(), 5);
        assert_eq!(CloneSlot(5).try_into_value(), Some(5));
        assert_eq!(CloneSlot(5).into_value(), 5);
        assert_eq!(CloneSlot(5).try_value(), Some(&5));
        assert_eq!(CloneSlot(6).try_value_mut(), Some(&mut 6));
        assert_eq!(CloneSlot(5).value(), &5);
        assert_eq!(CloneSlot(6).value_mut(), &mut 6);
        assert_eq!(CloneSlot(5).try_swap(Either::Left(7)), Some(5));
        let mut slot = CloneSlot(7);
        assert_eq!(slot.try_remove_value(), Some(7));
        assert_eq!(slot, CloneSlot(7));
        assert_eq!(slot.remove_value(), 7);
        assert_eq!(slot, CloneSlot(7));
        slot.delete_value();
        assert_eq!(slot, CloneSlot(7));
    }

    #[derive(PartialEq, Copy, Clone)]
    enum MySlot {
        Key(u8),
        Val(u16),
    }

    impl InitFrom<u16> for MySlot {
        fn from_value(value: u16) -> Self {
            MySlot::Val(value)
        }

        fn set_value(&mut self, new: u16) {
            *self = MySlot::Val(new)
        }
    }

    impl Slot for MySlot {
        type Value = u16;

        fn try_into_value(self) -> Option<u16> {
            match self {
                MySlot::Key(_) => None,
                MySlot::Val(v) => Some(v),
            }
        }
    }

    impl KeySlot<u8> for MySlot {
        fn try_key(&self) -> Option<u8> {
            match self {
                MySlot::Key(k) => Some(*k),
                MySlot::Val(_) => None,
            }
        }

        fn from_key(key: u8) -> Self {
            MySlot::Key(key)
        }
    }

    impl RemoveSlot for MySlot {
        fn try_remove_value(&mut self) -> Option<Self::Value> {
            match self {
                MySlot::Key(_) => None,
                MySlot::Val(v) => {
                    let v = *v;
                    *self = MySlot::Key(0);
                    Some(v)
                }
            }
        }
    }

    impl SlotRef for MySlot {
        fn try_value(&self) -> Option<&u16> {
            match self {
                MySlot::Key(_) => None,
                MySlot::Val(v) => Some(v),
            }
        }
    }

    impl SlotMut for MySlot {
        fn try_value_mut(&mut self) -> Option<&mut u16> {
            match self {
                MySlot::Key(_) => None,
                MySlot::Val(v) => Some(v),
            }
        }
    }

    impl CheckedSlot<u8> for MySlot {
        fn has_value(&self) -> bool {
            matches!(self, MySlot::Val(_))
        }

        fn has_key(&self) -> bool {
            matches!(self, MySlot::Key(_))
        }
    }

    #[test]
    fn default_impls() {
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
        assert_eq!(e.remove_value(), 15);
        assert_eq!(e.as_either(), Either::Left(0));
    }
}
