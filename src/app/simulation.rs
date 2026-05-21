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
use registers::{m_regular, safe_mrsw, safe_registers::safe_boolean_srsw};

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
