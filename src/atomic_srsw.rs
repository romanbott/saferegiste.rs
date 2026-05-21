use crate::m_regular::{MRegularMRSW, MRegularReader};

use crate::stamped_values::StampedValue;

pub struct AtomicSRSWReader {
    inner: MRegularReader,
    last_read: StampedValue<u16, 8>,
}

impl AtomicSRSWReader {
    pub fn read(&mut self) -> u8 {
        let value: StampedValue<u16, 8> = (self.inner.read() as u16).into();

        if value >= self.last_read {
            self.last_read.update(&value);
            value.value() as u8
        } else {
            self.last_read.value() as u8
        }
    }
}

pub struct AtomicSRSWWriter {
    inner: MRegularMRSW,
    last_stamp: u16,
}

impl AtomicSRSWWriter {
    pub fn write(&mut self, value: u8) {
        let new_stamp = self.last_stamp + 1;

        let stamped_value: StampedValue<u16, 8> = (new_stamp, value as u16).into();

        self.inner
            .write(stamped_value.value() as usize)
            .expect("Couldn write stamped byte.");
        self.last_stamp = new_stamp;
    }
}

pub fn new() -> (AtomicSRSWReader, AtomicSRSWWriter) {
    let mut m_reg = MRegularMRSW::new(1, 1 << 16);

    let m_reg_reader = m_reg.get_nth_reader(0).unwrap();

    (
        AtomicSRSWReader {
            inner: m_reg_reader,
            last_read: (0, 0).into(),
        },
        AtomicSRSWWriter {
            inner: m_reg,
            last_stamp: 0,
        },
    )
}

// fn main() {
//     let (mut r, mut w) = atomic_srsw();
//
//     let producer = thread::spawn(move || {
//         for i in 1..=7 {
//             println!("---> {}", i);
//             w.write(i);
//             thread::sleep(Duration::from_millis(500));
//         }
//     });
//
//     // Spawn the consumer thread
//     let consumer = thread::spawn(move || {
//         thread::sleep(Duration::from_millis(100));
//         loop {
//             let value = r.read();
//             println!("{} <--1", value);
//             thread::sleep(Duration::from_millis(100));
//             if value == 7 {
//                 break;
//             }
//         }
//     });
//
//     // Wait for both threads to complete their execution
//     producer.join().unwrap();
//     consumer.join().unwrap();
//
//     println!("All messages sent and received!");
// }
