use crate::{safe_mrsw::SafeMRSW, safe_registers::SafeReader};
use std::ops::Deref;

pub struct RegularMRSW {
    inner: SafeMRSW,
    last_written: bool,
}

pub struct RegularReader(SafeReader);

impl Deref for RegularReader {
    type Target = SafeReader;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
