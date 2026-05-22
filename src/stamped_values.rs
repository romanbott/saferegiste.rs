use std::cmp::Ordering;

/// A value that is packed together with a "stamp" (such as a version, timestamp, or sequence number)
/// into a single integer of type `T`.
///
/// The `VALUE_BITS` constant specifies how many of the least significant bits are reserved
/// for the value. The remaining most significant bits are used to store the stamp.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct StampedValue<T, const VALUE_BITS: usize> {
    inner: T,
}

// ==========================================
// u8 Implementation
// ==========================================

/// Creates a `StampedValue` directly from a raw, already-packed `u8`.
impl<const VALUE_BITS: usize> From<u8> for StampedValue<u8, VALUE_BITS> {
    fn from(value: u8) -> Self {
        StampedValue { inner: value }
    }
}

/// Creates a `StampedValue` from a `(stamp, value)` tuple.
///
/// # Panics
///
/// Panics if the `value` is too large to fit within the number of bits specified by `VALUE_BITS`.
impl<const VALUE_BITS: usize> From<(u8, u8)> for StampedValue<u8, VALUE_BITS> {
    fn from((stamp, value): (u8, u8)) -> Self {
        let mask = ((1_u16 << VALUE_BITS) - 1) as u8;
        assert!(value <= mask, "Value {} exceeds max {}", value, mask);

        StampedValue {
            inner: (stamp << VALUE_BITS) | value,
        }
    }
}

/// Compares two `StampedValue` instances based exclusively on their `stamp`.
///
/// The actual `value` payload is ignored during the comparison.
impl<const VALUE_BITS: usize> Ord for StampedValue<u8, VALUE_BITS> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp().cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> StampedValue<u8, VALUE_BITS> {
    /// Extracts the value portion of the packed integer.
    pub fn value(&self) -> u8 {
        let mask = ((1_u16 << VALUE_BITS) - 1) as u8;
        self.inner & mask
    }

    /// Extracts the stamp portion of the packed integer.
    pub fn stamp(&self) -> u8 {
        self.inner >> VALUE_BITS
    }

    /// Overwrites the current instance with the packed state of `other`.
    pub fn update(&mut self, other: &Self) {
        self.inner = other.inner;
    }

    /// Consumes the wrapper and returns the raw, packed `u8` containing both the stamp and the value.
    pub fn into_u8(&self) -> u8 {
        self.inner
    }
}

// ==========================================
// u16 Implementation
// ==========================================

/// Creates a `StampedValue` directly from a raw, already-packed `u16`.
impl<const VALUE_BITS: usize> From<u16> for StampedValue<u16, VALUE_BITS> {
    fn from(value: u16) -> Self {
        StampedValue { inner: value }
    }
}

/// Creates a `StampedValue` from a `(stamp, value)` tuple.
///
/// # Panics
///
/// Panics if the `value` is too large to fit within the number of bits specified by `VALUE_BITS`.
impl<const VALUE_BITS: usize> From<(u16, u16)> for StampedValue<u16, VALUE_BITS> {
    fn from((stamp, value): (u16, u16)) -> Self {
        let mask = ((1_u32 << VALUE_BITS) - 1) as u16;
        assert!(value <= mask, "Value {} exceeds max {}", value, mask);

        StampedValue {
            inner: (stamp << VALUE_BITS) | value,
        }
    }
}

/// Compares two `StampedValue` instances based exclusively on their `stamp`.
///
/// The actual `value` payload is ignored during the comparison.
impl<const VALUE_BITS: usize> Ord for StampedValue<u16, VALUE_BITS> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp().cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> StampedValue<u16, VALUE_BITS> {
    /// Extracts the value portion of the packed integer.
    pub fn value(&self) -> u16 {
        let mask = ((1_u32 << VALUE_BITS) - 1) as u16;
        self.inner & mask
    }

    /// Extracts the stamp portion of the packed integer.
    pub fn stamp(&self) -> u16 {
        self.inner >> VALUE_BITS
    }

    /// Overwrites the current instance with the packed state of `other`.
    pub fn update(&mut self, other: &Self) {
        self.inner = other.inner;
    }

    /// Returns the raw, packed `u16` containing both the stamp and the value.
    pub fn to_u16(&self) -> u16 {
        self.inner
    }
}
