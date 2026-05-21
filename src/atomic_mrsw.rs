use crate::{
    atomic_srsw::{self, AtomicSRSWReader, AtomicSRSWWriter},
    stamped_values::StampedValue,
};

struct AtomicMRSWReader {
    column: Vec<AtomicSRSWReader>,
    row: Vec<Option<AtomicSRSWWriter>>,
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
        let most_recent = self
            .column
            .iter_mut()
            .map(|srsw_reader| {
                let stamped_val: StampedValue<u8, 4> = srsw_reader.read().into();
                stamped_val
            })
            .max_by_key(|stamped_val| stamped_val.stamp())
            .expect("Couldn find most recent in reader column.");

        self.row.iter_mut().for_each(|maybe_writer| {
            if let Some(writer) = maybe_writer {
                writer.write(most_recent.into_u8());
            }
        });

        most_recent.value()
    }
}

struct AtomicMRSW {
    last_stamp: u8,
    readers: Vec<Option<AtomicMRSWReader>>,
    diagonal: Vec<AtomicSRSWWriter>,
}

impl AtomicMRSW {
    fn new(capacity: usize) -> Self {
        // For clarity, first build the matrix of atomic SRSW registers
        let mut matrix = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let mut row = Vec::with_capacity(capacity);
            for _ in 0..capacity {
                row.push(atomic_srsw::new());
            }
            matrix.push(row);
        }

        // Vector of readers
        let mut readers = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            readers.push(AtomicMRSWReader {
                column: vec![],
                row: vec![],
            });
        }

        // Diagonal (writer) entries for the writer
        let mut diagonal = Vec::with_capacity(capacity);

        // `into_iter` consumes the matrix, so we are following the SRSW rules.
        for (i, row) in matrix.into_iter().enumerate() {
            for (j, (r, w)) in row.into_iter().enumerate() {
                if i == j {
                    // Push the diagonal writer
                    diagonal.push(w);
                    // Push `None` as a place-holder, since each reader cant write to its own
                    // column.
                    readers[j].row.push(None);
                } else {
                    // Push the writer to the corresponding readers row
                    readers[j].row.push(Some(w));
                }
                // Push the reader to the corresponding readers column
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
        self.last_stamp += 1;
        let stamped_value: StampedValue<u8, 4> = (self.last_stamp, value).into();
        self.diagonal
            .iter_mut()
            .for_each(|d| d.write(stamped_value.into_u8()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_write_and_read_two_readers() {
        // Initialize the AtomicMRSW registry for 2 readers
        let mut mrsw = AtomicMRSW::new(2);

        // Extract both readers from the registry
        let mut reader_0 = mrsw.get_nth_reader(0).expect("Reader 0 should exist");
        let mut reader_1 = mrsw.get_nth_reader(1).expect("Reader 1 should exist");

        // Write the first value
        let first_value = 7;
        mrsw.write(first_value);

        // Verify both readers see the first value sequentially
        assert_eq!(
            reader_0.read(),
            first_value,
            "Reader 0 failed to read the first value"
        );
        assert_eq!(
            reader_1.read(),
            first_value,
            "Reader 1 failed to read the first value"
        );

        //  Write a second value to verify updates propagate correctly
        let second_value = 1;
        mrsw.write(second_value);

        // Verify both readers see the updated value sequentially
        assert_eq!(
            reader_1.read(),
            second_value,
            "Reader 1 failed to read the updated value"
        );
        assert_eq!(
            reader_0.read(),
            second_value,
            "Reader 0 failed to read the updated value"
        );
    }

    #[test]
    fn test_interleaved_reader_value_propagation() {
        let mut mrsw = AtomicMRSW::new(2);

        let mut reader_0 = mrsw.get_nth_reader(0).expect("Reader 0 should exist");
        let mut reader_1 = mrsw.get_nth_reader(1).expect("Reader 1 should exist");

        mrsw.write(10);

        assert_eq!(reader_0.read(), 10);

        mrsw.write(12);

        assert_eq!(reader_0.read(), 12);

        assert_eq!(reader_1.read(), 12);
    }

    #[test]
    fn test_cannot_grab_same_reader_twice() {
        let mut mrsw = AtomicMRSW::new(2);

        // First grab is successful
        assert!(mrsw.get_nth_reader(0).is_some());

        // Second grab returns None because Option::take left a None placeholder
        assert!(mrsw.get_nth_reader(0).is_none());
    }
}
