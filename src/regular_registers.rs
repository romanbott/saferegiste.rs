use crate::{safe_mrsw::SafeMRSW, safe_registers::SafeReader};
use std::ops::Deref;

pub struct RegularMRSW {
    inner: SafeMRSW,
    last_written: bool,
}

pub struct RegularReader(SafeReader);

// impl RegularReader {
//     pub fn new(safe_reader: SafeReader) -> Self {
//         Self(safe_reader)
//     }
//
//     pub fn into_inner(self) -> SafeReader {
//         self.0
//     }
// }
//
impl Deref for RegularReader {
    type Target = SafeReader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
//
// impl AsRef<SafeReader> for RegularReader {
//     fn as_ref(&self) -> &SafeReader {
//         &self.0
//     }
// }
//
// impl From<SafeReader> for RegularReader {
//     fn from(s: SafeReader) -> Self {
//         Self(s)
//     }
// }

impl RegularMRSW {
    pub fn new(capacity: usize) -> Self {
        RegularMRSW {
            inner: SafeMRSW::new(capacity),
            last_written: false,
        }
    }

    pub fn get_nth_reader(&mut self, n: usize) -> Option<RegularReader> {
        self.inner.get_nth_reader(n).map(RegularReader)
    }

    pub fn write(&mut self, value: bool) {
        if value != self.last_written {
            self.inner.write(value);
            self.last_written = value;
        }
    }
}

// fn main_1() {
//     let mut mrsw = RegularMRSW::new(2);
//
//     let first_reader = mrsw.get_nth_reader(0).unwrap();
//     let second_reader = mrsw.get_nth_reader(1).unwrap();
//
//     let producer = thread::spawn(move || {
//         let mut value = false;
//
//         for _ in 1..=10 {
//             println!("---> {}", value);
//             mrsw.write(value);
//             // value = !value;
//             thread::sleep(Duration::from_millis(1000));
//         }
//     });
//
//     // Spawn the consumer thread
//     let first_consumer = thread::spawn(move || {
//         thread::sleep(Duration::from_millis(100));
//         for _ in 1..=10 {
//             let value = first_reader.read();
//             println!("{} <--1", value);
//             thread::sleep(Duration::from_millis(1000));
//         }
//     });
//
//     let second_consumer = thread::spawn(move || {
//         thread::sleep(Duration::from_millis(200));
//         for _ in 1..=10 {
//             let value = second_reader.read();
//             println!("{} <--2", value);
//             thread::sleep(Duration::from_millis(1000));
//         }
//     });
//
//     // Wait for both threads to complete their execution
//     producer.join().unwrap();
//     first_consumer.join().unwrap();
//     second_consumer.join().unwrap();
//
//     println!("All messages sent and received!");
// }
