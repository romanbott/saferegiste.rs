use crate::atomic_srsw::{self, AtomicSRSWReader, AtomicSRSWWriter};

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
                let (r, w) = atomic_srsw::new();

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
