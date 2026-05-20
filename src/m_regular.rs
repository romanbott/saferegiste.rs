use crate::regular_registers::{RegularMRSW, RegularReader};

pub struct MRegularMRSW {
    inner: Vec<RegularMRSW>,
}

pub struct MRegularReader {
    inner: Vec<RegularReader>,
}

impl MRegularReader {
    pub fn read(&self) -> usize {
        self.inner.iter().position(|r| r.read()).unwrap()
    }
}

#[derive(Debug)]
enum WriterError {
    MValueExceeded,
}

impl MRegularMRSW {
    pub fn new(capacity: usize, m: usize) -> Self {
        let mut inner = Vec::with_capacity(m);

        for _ in 0..m {
            inner.push(RegularMRSW::new(capacity));
        }

        inner[0].write(true);

        MRegularMRSW { inner }
    }

    pub fn get_nth_reader(&mut self, n: usize) -> Option<MRegularReader> {
        let maybe_inner: Option<Vec<_>> = self
            .inner
            .iter_mut()
            .map(|regular| regular.get_nth_reader(n))
            .collect();

        maybe_inner.map(|inner| MRegularReader { inner })
    }

    pub fn write(&mut self, value: usize) -> Result<(), WriterError> {
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
