use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, Instant},
};

use crate::app::{RegisterType, SimEvent};

// Import the registers from your own library
use registers::{m_regular, safe_mrsw};

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
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    match reg_type {
        RegisterType::Safe => {
            let mut safe_reg = safe_mrsw::SafeMRSW::new(num_readers);
            let mut readers = vec![];
            for i in 0..num_readers {
                readers.push(safe_reg.get_nth_reader(i).unwrap());
            }

            let tx_writer = tx.clone();
            let writer_pause = pause_flag.clone();

            thread::spawn(move || {
                let mut current_val = false;
                for _ in 1..=10 {
                    smart_sleep(0, &writer_pause);
                    current_val = !current_val;

                    if tx_writer
                        .send(SimEvent::WriterUpdate(format!("Writing: {}", current_val)))
                        .is_err()
                    {
                        return;
                    }
                    safe_reg.write(current_val);
                    if tx_writer
                        .send(SimEvent::WriterUpdate(format!("Idle: {}", current_val)))
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
        RegisterType::MRegular => {
            // We set m = 16 because values 0..=15 require 16 underlying binary safe registers
            let mut mrsw = m_regular::MRegularMRSW::new(num_readers, 1 << 15);
            let mut readers = vec![];
            for i in 0..num_readers {
                readers.push(mrsw.get_nth_reader(i).unwrap());
            }

            let tx_writer = tx.clone();
            let writer_pause = pause_flag.clone();

            // Writer Thread
            thread::spawn(move || {
                for i in 0..=(1 << 8) {
                    // Loop from 0 up to 15 inclusive
                    smart_sleep(0, &writer_pause);

                    if tx_writer
                        .send(SimEvent::WriterUpdate(format!("Writing: {}", i)))
                        .is_err()
                    {
                        return;
                    }
                    let _ = mrsw.write(i);
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
        _ => {
            let _ = tx.send(SimEvent::Status("Pending integration.".to_string()));
        }
    }
}
