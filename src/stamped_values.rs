use std::cmp::Ordering;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct StampedByte<T, const VALUE_BITS: usize> {
    inner: T,
}

// ==========================================
// u8 Implementation
// ==========================================

impl<const VALUE_BITS: usize> From<u8> for StampedByte<u8, VALUE_BITS> {
    fn from(value: u8) -> Self {
        StampedByte { inner: value }
    }
}

impl<const VALUE_BITS: usize> From<(u8, u8)> for StampedByte<u8, VALUE_BITS> {
    fn from((stamp, value): (u8, u8)) -> Self {
        let mask = ((1_u16 << VALUE_BITS) - 1) as u8;
        assert!(value <= mask, "Value {} exceeds max {}", value, mask);

        StampedByte {
            inner: (stamp << VALUE_BITS) | value,
        }
    }
}

impl<const VALUE_BITS: usize> PartialOrd for StampedByte<u8, VALUE_BITS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stamp().partial_cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> Ord for StampedByte<u8, VALUE_BITS> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp().cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> StampedByte<u8, VALUE_BITS> {
    pub fn value(&self) -> u8 {
        let mask = ((1_u16 << VALUE_BITS) - 1) as u8;
        self.inner & mask
    }

    pub fn stamp(&self) -> u8 {
        self.inner >> VALUE_BITS
    }

    pub fn update(&mut self, other: &Self) {
        self.inner = other.inner;
    }
}

// ==========================================
// u16 Implementation
// ==========================================

impl<const VALUE_BITS: usize> From<u16> for StampedByte<u16, VALUE_BITS> {
    fn from(value: u16) -> Self {
        StampedByte { inner: value }
    }
}

impl<const VALUE_BITS: usize> From<(u16, u16)> for StampedByte<u16, VALUE_BITS> {
    fn from((stamp, value): (u16, u16)) -> Self {
        let mask = ((1_u32 << VALUE_BITS) - 1) as u16;
        assert!(value <= mask, "Value {} exceeds max {}", value, mask);

        StampedByte {
            inner: (stamp << VALUE_BITS) | value,
        }
    }
}

impl<const VALUE_BITS: usize> PartialOrd for StampedByte<u16, VALUE_BITS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.stamp().partial_cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> Ord for StampedByte<u16, VALUE_BITS> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stamp().cmp(&other.stamp())
    }
}

impl<const VALUE_BITS: usize> StampedByte<u16, VALUE_BITS> {
    pub fn value(&self) -> u16 {
        let mask = ((1_u32 << VALUE_BITS) - 1) as u16;
        self.inner & mask
    }

    pub fn stamp(&self) -> u16 {
        self.inner >> VALUE_BITS
    }

    pub fn update(&mut self, other: &Self) {
        self.inner = other.inner;
    }
}
