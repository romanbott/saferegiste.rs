use std::thread;
use std::time::Duration;

mod atomic_mrsw;
mod atomic_srsw;
mod m_regular;
mod regular_registers;
mod safe_mrsw;
mod safe_registers;
mod stamped_values;

// #[derive(PartialEq, Eq)]
// struct StampedByte {
//     inner: u16,
// }
//
// impl From<u16> for StampedByte {
//     fn from(value: u16) -> Self {
//         StampedByte { inner: value }
//     }
// }
//
// impl From<(u16, u8)> for StampedByte {
//     fn from((stamp, value): (u16, u8)) -> Self {
//         // TODO: add validation so value is not greater than 7
//         StampedByte {
//             inner: stamp << 3 | value as u16,
//         }
//     }
// }
//
// impl PartialOrd for StampedByte {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.stamp().partial_cmp(&other.stamp())
//     }
// }
//
// impl Ord for StampedByte {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.stamp().cmp(&other.stamp())
//     }
// }
//
// impl StampedByte {
//     fn value(&self) -> u8 {
//         (self.inner & 7) as u8
//     }
//
//     fn stamp(&self) -> u16 {
//         self.inner >> 3
//     }
//
//     fn update(&mut self, other: &StampedByte) {
//         self.inner = other.inner
//     }
// }
