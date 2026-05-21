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
pub enum WriterError {
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

        // Write first, so theres always at least one `true` in the array
        self.inner[value].write(true);

        // Now set all the "downstream" entries to `false`
        for reg in self.inner[..value].iter_mut().rev() {
            reg.write(false);
        }
        Ok(())
    }
}
