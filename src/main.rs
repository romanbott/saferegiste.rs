use std::thread;
use std::time::Duration;

mod safe_registers;
mod stamped_values;

use safe_registers::{Reader, Writer, boolean_srsw};

fn main2() {
    let (r, mut w) = boolean_srsw();

    let producer = thread::spawn(move || {
        let mut value = false;

        for _ in 1..=10 {
            println!("---> {}", value);
            w.write(value);
            value = !value;
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Spawn the consumer thread
    let consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        for _ in 1..=10 {
            let value = r.read();
            println!("{} <---", value);
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Wait for both threads to complete their execution
    producer.join().unwrap();
    consumer.join().unwrap();

    println!("All messages sent and received!");
}

fn main_1() {
    let mut mrsw = RegularMRSW::new(2);

    let first_reader = mrsw.get_nth_reader(0).unwrap();
    let second_reader = mrsw.get_nth_reader(1).unwrap();

    let producer = thread::spawn(move || {
        let mut value = false;

        for _ in 1..=10 {
            println!("---> {}", value);
            mrsw.write(value);
            // value = !value;
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Spawn the consumer thread
    let first_consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        for _ in 1..=10 {
            let value = first_reader.read();
            println!("{} <--1", value);
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let second_consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        for _ in 1..=10 {
            let value = second_reader.read();
            println!("{} <--2", value);
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Wait for both threads to complete their execution
    producer.join().unwrap();
    first_consumer.join().unwrap();
    second_consumer.join().unwrap();

    println!("All messages sent and received!");
}

struct MRSW {
    readers: Vec<Option<Reader>>,
    writers: Vec<Writer>,
}

impl MRSW {
    fn new(capacity: usize) -> Self {
        let mut readers = Vec::with_capacity(capacity);
        let mut writers = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let (r, w) = boolean_srsw();

            readers.push(Some(r));
            writers.push(w);
        }

        MRSW { readers, writers }
    }

    fn get_nth_reader(&mut self, n: usize) -> Option<Reader> {
        self.readers.get_mut(n).and_then(Option::take)
    }

    fn write(&mut self, value: bool) {
        self.writers.iter_mut().for_each(|w| w.write(value));
    }
}

struct RegularMRSW {
    inner: MRSW,
    last_written: bool,
}

impl RegularMRSW {
    fn new(capacity: usize) -> Self {
        RegularMRSW {
            inner: MRSW::new(capacity),
            last_written: false,
        }
    }

    fn get_nth_reader(&mut self, n: usize) -> Option<Reader> {
        self.inner.get_nth_reader(n)
    }

    fn write(&mut self, value: bool) {
        if value != self.last_written {
            self.inner.write(value);
            self.last_written = value;
        }
    }
}

struct MRegularMRSW {
    inner: Vec<RegularMRSW>,
}

struct MRegularReader {
    inner: Vec<Reader>,
}

impl MRegularReader {
    fn read(&self) -> usize {
        self.inner.iter().position(|r| r.read()).unwrap()
    }
}

#[derive(Debug)]
enum WriterError {
    MValueExceeded,
}

impl MRegularMRSW {
    fn new(capacity: usize, m: usize) -> Self {
        let mut inner = Vec::with_capacity(m);

        for _ in 0..m {
            inner.push(RegularMRSW::new(capacity));
        }

        inner[0].write(true);

        MRegularMRSW { inner }
    }

    fn get_nth_reader(&mut self, n: usize) -> Option<MRegularReader> {
        let maybe_inner: Option<Vec<_>> = self
            .inner
            .iter_mut()
            .map(|regular| regular.get_nth_reader(n))
            .collect();

        maybe_inner.map(|inner| MRegularReader { inner })
    }

    fn write(&mut self, value: usize) -> Result<(), WriterError> {
        if value >= self.inner.len() {
            return Err(WriterError::MValueExceeded);
        }

        for (n, reg) in self.inner.iter_mut().enumerate().rev() {
            if n == value {
                reg.write(true);
            } else {
                reg.write(false);
            }
        }
        Ok(())
    }
}

fn main_m_regular() {
    let mut mrsw = MRegularMRSW::new(2, 11);

    let first_reader = mrsw.get_nth_reader(0).unwrap();
    let second_reader = mrsw.get_nth_reader(1).unwrap();

    let producer = thread::spawn(move || {
        for i in 1..=10 {
            println!("---> {}", i);
            mrsw.write(i).expect("Valor excedido");
            thread::sleep(Duration::from_millis(500));
        }
    });

    // Spawn the consumer thread
    let first_consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        loop {
            let value = first_reader.read();
            println!("{} <--1", value);
            thread::sleep(Duration::from_millis(200));
            if value == 10 {
                break;
            }
        }
    });

    let second_consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        loop {
            let value = second_reader.read();
            println!("{} <--2", value);
            thread::sleep(Duration::from_millis(200));
            if value == 10 {
                break;
            }
        }
    });

    // Wait for both threads to complete their execution
    producer.join().unwrap();
    first_consumer.join().unwrap();
    second_consumer.join().unwrap();

    println!("All messages sent and received!");
}

struct AtomicSRSWReader {
    inner: MRegularReader,
    last_read: StampedByte,
}

// public T read() {
// StampedValue<T> value = r_value;
// StampedValue<T> last = lastRead.get();
// StampedValue<T> result = StampedValue.max(value, last);
// lastRead.set(result);
// return result.value;
// }
impl AtomicSRSWReader {
    fn read(&mut self) -> u8 {
        let value: StampedByte = (self.inner.read() as u16).into();

        if &value >= &self.last_read {
            self.last_read.update(&value);
            return value.value();
        } else {
            return self.last_read.value();
        }
    }
}

// public void write(T v) {
// long stamp = lastStamp.get() + 1;
// r_value = new StampedValue(stamp, v);
// lastStamp.set(stamp);

struct AtomicSRSWWriter {
    inner: MRegularMRSW,
    last_stamp: u16,
}

impl AtomicSRSWWriter {
    fn write(&mut self, value: u8) {
        let new_stamp = self.last_stamp + 1;

        let stamped_value: StampedByte = (new_stamp, value).into();

        self.inner
            .write(stamped_value.inner as usize)
            .expect("Couldn write stamped byte.");
        self.last_stamp = new_stamp;
    }
}

fn atomic_srsw() -> (AtomicSRSWReader, AtomicSRSWWriter) {
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

#[derive(PartialEq, Eq)]
struct StampedByte {
    inner: u16,
}

impl From<u16> for StampedByte {
    fn from(value: u16) -> Self {
        StampedByte { inner: value }
    }
}

impl From<(u16, u8)> for StampedByte {
    fn from((stamp, value): (u16, u8)) -> Self {
        // TODO: add validation so value is not greater than 7
        StampedByte {
            inner: stamp << 3 | value as u16,
        }
    }
}

impl PartialOrd for StampedByte {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.stamp().partial_cmp(&other.stamp())
    }
}

impl Ord for StampedByte {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.stamp().cmp(&other.stamp())
    }
}

impl StampedByte {
    fn value(&self) -> u8 {
        (self.inner & 7) as u8
    }

    fn stamp(&self) -> u16 {
        self.inner >> 3
    }

    fn update(&mut self, other: &StampedByte) {
        self.inner = other.inner
    }
}

fn main() {
    let (mut r, mut w) = atomic_srsw();

    let producer = thread::spawn(move || {
        for i in 1..=7 {
            println!("---> {}", i);
            w.write(i);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // Spawn the consumer thread
    let consumer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        loop {
            let value = r.read();
            println!("{} <--1", value);
            thread::sleep(Duration::from_millis(100));
            if value == 7 {
                break;
            }
        }
    });

    // Wait for both threads to complete their execution
    producer.join().unwrap();
    consumer.join().unwrap();

    println!("All messages sent and received!");
}

struct AtomicMRSWReader {
    column: Vec<AtomicSRSWReader>,
    row: Vec<AtomicSRSWWriter>,
}

// public class AtomicMRSWRegister<T> implements Register<T> {
// 	ThreadLocal<Long> lastStamp;
// 	private StampedValue<T>[][] a_table; // each entry is an atomic SRSW register
//
// 	public AtomicMRSWRegister(T init, int readers) {
// 		lastStamp = new ThreadLocal<Long>() {
// 			protected Long initialValue() {
// 				return 0;
// 			};
// 		};
// 		a_table = (StampedValue<T>[][]) new StampedValue[readers][readers];
// 		StampedValue<T> value = new StampedValue<T>(init);
// 		for (int i = 0; i < readers; i++) {
// 			for (int j = 0; j < readers; j++) {
// 				a_table[i][j] = value;
// 			}
// 		}
// 	}
//
// 	public T read() {
// 		int me = ThreadID.get();
// 		StampedValue<T> value = a_table[me][me];
// 		for (int i = 0; i < a_table.length; i++) {
// 			value = StampedValue.max(value, a_table[i][me]);
// 		}
// 		for (int i = 0; i < a_table.length; i++) {
// 			if (i == me)
// 				continue;
// 			a_table[me][i] = value;
// 		}
// 		return value;
// 	}
//
// }

impl AtomicMRSWReader {
    fn read(&mut self) -> u8 {
        todo!()
    }
}

struct AtomicMRSW {
    last_stamp: u16,
    readers: Vec<Option<AtomicMRSWReader>>,
    diagonal: Vec<AtomicSRSWWriter>,
}

impl AtomicMRSW {
    fn new(capacity: usize) -> Self {
        let mut readers = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            readers.push(AtomicMRSWReader {
                column: vec![],
                row: vec![],
            });
        }

        let mut diagonal = Vec::with_capacity(capacity);

        for i in 0..capacity {
            for j in 0..capacity {
                let (r, w) = atomic_srsw();

                if i == j {
                    diagonal.push(w);
                } else {
                    readers[j].row.push(w);
                }
                readers[j].column.push(r);
            }
        }

        let readers = readers.into_iter().map(Option::Some).collect();

        AtomicMRSW {
            last_stamp: 0,
            readers,
            diagonal,
        }
    }

    fn get_nth_reader(&mut self, n: usize) -> Option<AtomicMRSWReader> {
        self.readers.get_mut(n).and_then(Option::take)
    }

    // 	public void write(T v) {
    // 		long stamp = lastStamp.get() + 1;
    // 		lastStamp.set(stamp);
    // 		StampedValue<T> value = new StampedValue<T>(stamp, v);
    // 		for (int i = 0; i < a_table.length; i++) {
    // 			a_table[i][i] = value;
    // 		}
    // 	}

    fn write(&mut self, value: u8) {
        self.diagonal.iter_mut().for_each(|d| d.write(value));
    }
}
