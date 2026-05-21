use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::Duration;

pub static FLICKER_MODE: AtomicU8 = AtomicU8::new(1); // Default to Normal

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
        let mode = FLICKER_MODE.load(Ordering::Relaxed);
        let (iters, sleep) = match mode {
            0 => (2..=5, 1..=5),    // Fast
            1 => (5..=10, 5..=25),  // Normal
            _ => (5..=10, 10..=50), // Slow
        };

        unsafe {
            // 1. Get the raw const pointer to the inner data
            let const_ptr = Arc::as_ptr(&self.inner);

            // 2. Cast it to a mutable pointer
            let mut_ptr = const_ptr as *mut bool;

            // 3. Dereference and mutate

            let iterations = rand::random_range(iters);

            for _ in 0..iterations {
                // 2. Generate a random sleep duration in milliseconds (e.g., 100ms to 500ms)
                let sleep_time_ms = rand::random_range(sleep.clone());

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
