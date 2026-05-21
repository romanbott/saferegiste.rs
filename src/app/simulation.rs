use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, Instant},
};

use crate::app::{BOOLEAN_SEQUENCES, NUMERIC_SEQUENCES, RegisterType, SimEvent};

// Import the registers from your own library
use registers::{
    atomic_mrmw::AtomicMRMW, atomic_mrsw, atomic_srsw, m_regular, safe_mrsw,
    safe_registers::safe_boolean_srsw,
};

pub fn smart_sleep(delay_ms: u64, pause_flag: &Arc<AtomicBool>) {
    let target = Duration::from_millis(delay_ms);
    let start = Instant::now();

    loop {
        while pause_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }
        if start.elapsed() >= target {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn run_simulation(
    reg_type: RegisterType,
    num_readers: usize,
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    match reg_type {
        RegisterType::SafeSRSW => simulate_safe_srsw(
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),
        RegisterType::SafeMRSW => simulate_safe_mrsw(
            num_readers,
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),

        RegisterType::Regular => simulate_regular(
            num_readers,
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),

        RegisterType::MRegular => simulate_m_regular(
            num_readers,
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),
        RegisterType::AtomicSRSW => simulate_atomic_srsw(
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),
        RegisterType::AtomicMRSW => simulate_atomic_mrsw(
            num_readers,
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),
        RegisterType::AtomicMRMW => simulate_atomic_mrmw(
            num_readers,
            num_reads,
            writer_delay_ms,
            reader_delay_ms,
            seq_idx,
            tx,
            pause_flag,
        ),

        _ => {
            let _ = tx.send(SimEvent::Status("Pending integration.".to_string()));
        }
    }
}

fn simulate_safe_srsw(
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    let (reader, mut writer) = safe_boolean_srsw();

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = BOOLEAN_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for val in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", val)))
                .is_err()
            {
                return;
            }
            writer.write(val);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", val)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    let tx_reader = tx.clone();
    let reader_pause = pause_flag.clone();

    thread::spawn(move || {
        smart_sleep(100, &reader_pause);
        for _ in 1..=num_reads {
            let value = reader.read();
            if tx_reader
                .send(SimEvent::ReaderUpdate(0, format!("{}", value)))
                .is_err()
            {
                return;
            }
            smart_sleep(reader_delay_ms, &reader_pause);
        }
    });
}

fn simulate_safe_mrsw(
    num_readers: usize,
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    let mut safe_reg = safe_mrsw::SafeMRSW::new(num_readers);
    let mut readers = vec![];
    for i in 0..num_readers {
        readers.push(safe_reg.get_nth_reader(i).unwrap());
    }

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = BOOLEAN_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for val in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", val)))
                .is_err()
            {
                return;
            }
            safe_reg.write(val);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", val)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    for (id, reader) in readers.into_iter().enumerate() {
        let tx_reader = tx.clone();
        let reader_pause = pause_flag.clone();

        thread::spawn(move || {
            smart_sleep(100, &reader_pause);
            for _ in 1..=num_reads {
                let value = reader.read();
                if tx_reader
                    .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                    .is_err()
                {
                    return;
                }
                smart_sleep(reader_delay_ms, &reader_pause);
            }
        });
    }
}

fn simulate_regular(
    num_readers: usize,
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    let mut mrsw = registers::regular_registers::RegularMRSW::new(num_readers);

    let mut readers = vec![];
    for i in 0..num_readers {
        readers.push(mrsw.get_nth_reader(i).unwrap());
    }

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = BOOLEAN_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for val in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", val)))
                .is_err()
            {
                return;
            }
            let _ = mrsw.write(val);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", val)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    // Reader Threads
    for (id, reader) in readers.into_iter().enumerate() {
        let tx_reader = tx.clone();
        let reader_pause = pause_flag.clone();
        thread::spawn(move || {
            smart_sleep(100, &reader_pause); // Stagger start slightly
            for _ in 1..=num_reads {
                let value = reader.read();
                if tx_reader
                    .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                    .is_err()
                {
                    return;
                }
                smart_sleep(reader_delay_ms, &reader_pause);
            }
        });
    }
}

fn simulate_m_regular(
    num_readers: usize,
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    // We set m = 16 because values 0..=15 require 16 underlying binary safe registers
    let mut mrsw = m_regular::MRegularMRSW::new(num_readers, 1 << 15);
    let mut readers = vec![];
    for i in 0..num_readers {
        readers.push(mrsw.get_nth_reader(i).unwrap());
    }

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = NUMERIC_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for i in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", i)))
                .is_err()
            {
                return;
            }
            let _ = mrsw.write(i as usize);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", i)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    // Reader Threads
    for (id, reader) in readers.into_iter().enumerate() {
        let tx_reader = tx.clone();
        let reader_pause = pause_flag.clone();
        thread::spawn(move || {
            smart_sleep(100, &reader_pause); // Stagger start slightly
            for _ in 1..=num_reads {
                let value = reader.read();
                if tx_reader
                    .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                    .is_err()
                {
                    return;
                }
                smart_sleep(reader_delay_ms, &reader_pause);
            }
        });
    }
}

fn simulate_atomic_srsw(
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    let (mut reader, mut writer) = atomic_srsw::new();

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = NUMERIC_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for i in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", i)))
                .is_err()
            {
                return;
            }
            let _ = writer.write(i);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", i)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    // Reader Threads
    let tx_reader = tx.clone();
    let reader_pause = pause_flag.clone();
    thread::spawn(move || {
        smart_sleep(100, &reader_pause); // Stagger start slightly
        for _ in 1..=num_reads {
            let value = reader.read();
            if tx_reader
                .send(SimEvent::ReaderUpdate(0, format!("{}", value)))
                .is_err()
            {
                return;
            }
            smart_sleep(reader_delay_ms, &reader_pause);
        }
    });
}

fn simulate_atomic_mrsw(
    num_readers: usize,
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    // We set m = 16 because values 0..=15 require 16 underlying binary safe registers
    let mut mrsw = atomic_mrsw::AtomicMRSW::new(num_readers);
    let mut readers = vec![];
    for i in 0..num_readers {
        readers.push(mrsw.get_nth_reader(i).unwrap());
    }

    let tx_writer = tx.clone();
    let writer_pause = pause_flag.clone();

    let sequence = NUMERIC_SEQUENCES[seq_idx].to_vec();

    thread::spawn(move || {
        for i in sequence.into_iter().cycle() {
            smart_sleep(0, &writer_pause);

            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Writing: {}", i)))
                .is_err()
            {
                return;
            }
            let _ = mrsw.write(i as u8);
            if tx_writer
                .send(SimEvent::WriterUpdate(format!("Idle: {}", i)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &writer_pause);
        }
        let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
    });

    // Reader Threads
    for (id, mut reader) in readers.into_iter().enumerate() {
        let tx_reader = tx.clone();
        let reader_pause = pause_flag.clone();
        thread::spawn(move || {
            smart_sleep(100, &reader_pause); // Stagger start slightly
            for _ in 1..=num_reads {
                let value = reader.read();
                if tx_reader
                    .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                    .is_err()
                {
                    return;
                }
                smart_sleep(reader_delay_ms, &reader_pause);
            }
        });
    }
}

fn simulate_atomic_mrmw(
    _num_readers: usize, // Ignored since we are using a fixed 2-reader/2-writer topology
    num_reads: usize,
    writer_delay_ms: u64,
    reader_delay_ms: u64,
    seq_idx: usize,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    let writer_delay_ms = 10000.max(writer_delay_ms);

    // 1. Initialize an AtomicMRMW register with a fixed capacity of 4 nodes
    let mut mrmw = AtomicMRMW::new(4);

    // 2. Extract 4 separate reader-writer handles
    let mut writer1 = mrmw.get_nth_reader(0).expect("Writer 1 node missing");
    let mut writer2 = mrmw.get_nth_reader(1).expect("Writer 2 node missing");
    let mut reader1 = mrmw.get_nth_reader(2).expect("Reader 1 node missing");
    let mut reader2 = mrmw.get_nth_reader(3).expect("Reader 2 node missing");

    // Get the chosen numeric sequence from global app configurations
    let sequence = NUMERIC_SEQUENCES[seq_idx].to_vec();

    // -------------------------------------------------------------------------
    // WRITER THREAD 1
    // -------------------------------------------------------------------------
    let tx_w1 = tx.clone();
    let pause_w1 = pause_flag.clone();
    let seq_w1 = sequence.clone(); // Clone the sequence vector for Writer 1
    thread::spawn(move || {
        for i in seq_w1.into_iter().cycle() {
            smart_sleep(0, &pause_w1);

            if tx_w1
                .send(SimEvent::WriterUpdate(format!("W1 Writing: {}", i)))
                .is_err()
            {
                return;
            }
            writer1.write(i as u8);
            if tx_w1
                .send(SimEvent::WriterUpdate(format!("W1 Idle: {}", i)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &pause_w1);
        }
    });

    // -------------------------------------------------------------------------
    // WRITER THREAD 2
    // -------------------------------------------------------------------------
    let tx_w2 = tx.clone();
    let pause_w2 = pause_flag.clone();
    let seq_w2 = sequence.clone(); // Clone the sequence vector for Writer 2
    thread::spawn(move || {
        // Stagger Writer 2 slightly so they don't overlap initial writes instantly
        smart_sleep(writer_delay_ms / 2, &pause_w2);
        for i in seq_w2.into_iter().cycle() {
            let i = (i + 3) % 8;
            smart_sleep(0, &pause_w2);

            if tx_w2
                .send(SimEvent::WriterUpdate(format!("W2 Writing: {}", i)))
                .is_err()
            {
                return;
            }
            writer2.write(i as u8);
            if tx_w2
                .send(SimEvent::WriterUpdate(format!("W2 Idle: {}", i)))
                .is_err()
            {
                return;
            }

            smart_sleep(writer_delay_ms, &pause_w2);
        }
    });

    // -------------------------------------------------------------------------
    // READER THREAD 1 (Maps to UI row index 0)
    // -------------------------------------------------------------------------
    let tx_r1 = tx.clone();
    let pause_r1 = pause_flag.clone();
    thread::spawn(move || {
        smart_sleep(100, &pause_r1); // Stagger start slightly
        for _ in 1..=num_reads {
            let value = reader1.read();
            if tx_r1
                .send(SimEvent::ReaderUpdate(0, format!("{}", value)))
                .is_err()
            {
                return;
            }
            smart_sleep(reader_delay_ms, &pause_r1);
        }
    });

    // -------------------------------------------------------------------------
    // READER THREAD 2 (Maps to UI row index 1)
    // -------------------------------------------------------------------------
    let tx_r2 = tx.clone();
    let pause_r2 = pause_flag.clone();
    thread::spawn(move || {
        smart_sleep(100, &pause_r2); // Stagger start slightly
        for _ in 1..=num_reads {
            let value = reader2.read();
            if tx_r2
                .send(SimEvent::ReaderUpdate(1, format!("{}", value)))
                .is_err()
            {
                return;
            }
            smart_sleep(reader_delay_ms, &pause_r2);
        }
    });
}
