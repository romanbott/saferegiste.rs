use crate::safe_registers::{SafeReader, SafeWriter, safe_boolean_srsw};

pub struct SafeMRSW {
    readers: Vec<Option<SafeReader>>,
    writers: Vec<SafeWriter>,
}

impl SafeMRSW {
    pub fn new(capacity: usize) -> Self {
        let mut readers = Vec::with_capacity(capacity);
        let mut writers = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let (r, w) = safe_boolean_srsw();

            readers.push(Some(r));
            writers.push(w);
        }

        SafeMRSW { readers, writers }
    }

    pub fn get_nth_reader(&mut self, n: usize) -> Option<SafeReader> {
        self.readers.get_mut(n).and_then(Option::take)
    }

    pub fn write(&mut self, value: bool) {
        self.writers.iter_mut().for_each(|w| w.write(value));
    }
}
