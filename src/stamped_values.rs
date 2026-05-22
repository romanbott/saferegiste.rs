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

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // u8 Tests
    // ==========================================

    #[test]
    fn test_u8_packing_and_unpacking() {
        // 4 bits for value, 4 bits for stamp
        let stamped: StampedValue<u8, 4> = (3, 5).into(); // stamp: 3, value: 5

        assert_eq!(stamped.stamp(), 3);
        assert_eq!(stamped.value(), 5);

        // Raw representation: (3 << 4) | 5 = 48 | 5 = 53
        assert_eq!(stamped.into_u8(), 53);

        let from_raw: StampedValue<u8, 4> = 53.into();
        assert_eq!(from_raw.stamp(), 3);
        assert_eq!(from_raw.value(), 5);
    }

    #[test]
    fn test_u8_ordering() {
        // Ordering should rely entirely on the stamp, ignoring the value.
        let low_stamp_high_val: StampedValue<u8, 4> = (1, 15).into();
        let high_stamp_low_val: StampedValue<u8, 4> = (2, 0).into();
        let same_stamp_diff_val: StampedValue<u8, 4> = (1, 0).into();

        // 1 < 2, despite value being 15 vs 0
        assert!(low_stamp_high_val < high_stamp_low_val);
        // Stamps are equal, so the ordering evaluates to Equal
        assert_eq!(
            low_stamp_high_val.cmp(&same_stamp_diff_val),
            Ordering::Equal
        );
    }

    #[test]
    #[should_panic(expected = "exceeds max")]
    fn test_u8_value_overflow_panics() {
        // 4 bits for value means max value is 15. Passing 16 should panic.
        let _stamped: StampedValue<u8, 4> = (1, 16).into();
    }

    #[test]
    fn test_u8_update() {
        let mut val1: StampedValue<u8, 4> = (1, 5).into();
        let val2: StampedValue<u8, 4> = (2, 10).into();

        val1.update(&val2);

        assert_eq!(val1.stamp(), 2);
        assert_eq!(val1.value(), 10);
    }

    // ==========================================
    // u16 Tests
    // ==========================================

    #[test]
    fn test_u16_packing_and_unpacking() {
        // 10 bits for value (max 1023), 6 bits for stamp (max 63)
        let stamped: StampedValue<u16, 10> = (5, 1000).into();

        assert_eq!(stamped.stamp(), 5);
        assert_eq!(stamped.value(), 1000);

        // Raw representation: (5 << 10) | 1000 = 5120 | 1000 = 6120
        assert_eq!(stamped.to_u16(), 6120);

        let from_raw: StampedValue<u16, 10> = 6120.into();
        assert_eq!(from_raw.stamp(), 5);
        assert_eq!(from_raw.value(), 1000);
    }

    #[test]
    fn test_u16_ordering() {
        let low_stamp_high_val: StampedValue<u16, 10> = (10, 1023).into();
        let high_stamp_low_val: StampedValue<u16, 10> = (11, 0).into();
        let same_stamp_diff_val: StampedValue<u16, 10> = (10, 500).into();

        assert!(low_stamp_high_val < high_stamp_low_val);
        assert_eq!(
            low_stamp_high_val.cmp(&same_stamp_diff_val),
            Ordering::Equal
        );
    }

    #[test]
    #[should_panic(expected = "exceeds max")]
    fn test_u16_value_overflow_panics() {
        // 10 bits for value means max value is 1023. Passing 1024 should panic.
        let _stamped: StampedValue<u16, 10> = (1, 1024).into();
    }

    #[test]
    fn test_u16_update() {
        let mut val1: StampedValue<u16, 12> = (5, 4000).into();
        let val2: StampedValue<u16, 12> = (6, 4001).into();

        val1.update(&val2);

        assert_eq!(val1.stamp(), 6);
        assert_eq!(val1.value(), 4001);
    }
}
