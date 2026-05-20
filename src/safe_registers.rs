use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct NoCopyBool(bool);

pub struct SafeReader {
    inner: Arc<NoCopyBool>,
}

pub struct SafeWriter {
    inner: Arc<NoCopyBool>,
}

impl SafeReader {
    pub fn read(&self) -> bool {
        self.inner.deref().0
    }
}

impl SafeWriter {
    pub fn write(&mut self, value: bool) {
        unsafe {
            // 1. Get the raw const pointer to the inner data
            let const_ptr = Arc::as_ptr(&self.inner);

            // 2. Cast it to a mutable pointer
            let mut_ptr = const_ptr as *mut bool;

            // 3. Dereference and mutate

            let iterations = rand::random_range(5..=10);

            for _ in 0..iterations {
                // 2. Generate a random sleep duration in milliseconds (e.g., 100ms to 500ms)
                let sleep_time_ms = rand::random_range(10..=50);

                thread::sleep(Duration::from_millis(sleep_time_ms));

                *mut_ptr = rand::random_bool(0.5);
            }

            *mut_ptr = value;
        }
    }
}

pub fn safe_boolean_srsw() -> (SafeReader, SafeWriter) {
    let inner = Arc::new(NoCopyBool(false));

    (
        SafeReader {
            inner: inner.clone(),
        },
        SafeWriter { inner },
    )
}
