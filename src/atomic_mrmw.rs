use crate::{
    atomic_mrsw::{AtomicMRSW, AtomicMRSWReader},
    stamped_values::StampedValue,
};

pub struct AtomicMRMWReaderWriter {
    writer: AtomicMRSW,
    readers: Vec<AtomicMRSWReader>,
}

impl AtomicMRMWReaderWriter {
    pub fn read(&mut self) -> u8 {
        let most_recent = self
            .readers
            .iter_mut()
            .map(|srsw_reader| {
                let stamped_val: StampedValue<u8, 3> = srsw_reader.read().into();
                stamped_val
            })
            .max_by_key(|stamped_val| stamped_val.stamp())
            .expect("Couldn find most recent in reader.");

        most_recent.value()
    }

    pub fn write(&mut self, value: u8) {
        let most_recent_stamp = self
            .readers
            .iter_mut()
            .map(|srsw_reader| {
                let stamped_val: StampedValue<u8, 3> = srsw_reader.read().into();
                stamped_val.stamp()
            })
            .max()
            .expect("Couldn find most recent in reader.");

        let stamped_val: StampedValue<u8, 3> = (most_recent_stamp + 1, value).into();

        self.writer.write(stamped_val.into_u8());
    }
}

pub struct AtomicMRMW {
    readers_writers: Vec<Option<AtomicMRMWReaderWriter>>,
}

impl AtomicMRMW {
    pub fn new(capacity: usize) -> AtomicMRMW {
        let mut mrsws = Vec::new();

        for _ in 0..capacity {
            mrsws.push(AtomicMRSW::new(capacity));
        }

        let mut readers_rows = Vec::new();

        for _ in 0..capacity {
            readers_rows.push(Vec::new());
        }

        for i in 0..capacity {
            for mrsw in mrsws.iter_mut() {
                readers_rows[i].push(mrsw.get_nth_reader(i).expect("Reader already taken!."));
            }
        }

        let readers_writers = mrsws
            .into_iter()
            .zip(readers_rows)
            .map(|(mrsw, rr)| {
                Some(AtomicMRMWReaderWriter {
                    writer: mrsw,
                    readers: rr,
                })
            })
            .collect();

        AtomicMRMW { readers_writers }
    }

    pub fn get_nth_reader(&mut self, n: usize) -> Option<AtomicMRMWReaderWriter> {
        self.readers_writers.get_mut(n).and_then(Option::take)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_mrmw_two_nodes() {
        // 1. Initialize the MRMW system for 2 participants
        let mut mrmw = AtomicMRMW::new(2);

        // 2. Extract both reader/writer handles
        let mut node_0 = mrmw.get_nth_reader(0).expect("Node 0 should exist");
        let mut node_1 = mrmw.get_nth_reader(1).expect("Node 1 should exist");

        // --- FIRST WRITE CYCLE ---
        // Node 0 acts as the writer and writes 3
        node_0.write(3);

        // Both nodes act as readers and verify the value is 3
        assert_eq!(
            node_0.read(),
            3,
            "Node 0 failed to read its own written value"
        );
        assert_eq!(
            node_1.read(),
            3,
            "Node 1 failed to read the value written by Node 0"
        );

        // --- SECOND WRITE CYCLE ---
        // Node 1 acts as the writer and updates the value to 7
        node_1.write(7);

        // Both nodes act as readers and verify the system has moved to 7
        assert_eq!(
            node_1.read(),
            7,
            "Node 1 failed to read its own updated value"
        );
        assert_eq!(
            node_0.read(),
            7,
            "Node 0 failed to see the updated value from Node 1"
        );
    }

    #[test]
    fn test_cannot_double_extract_nodes() {
        let mut mrmw = AtomicMRMW::new(2);

        // First pull is successful
        assert!(mrmw.get_nth_reader(0).is_some());

        // Second pull is None because Option::take left a None placeholder behind
        assert!(mrmw.get_nth_reader(0).is_none());
    }
}
