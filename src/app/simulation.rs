use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
    time::{Duration, Instant},
};

use registers::{m_regular::MRegularMRSW, safe_mrsw::SafeMRSW};

use crate::app::{RegisterType, SimEvent};

// -------------------------------------------------------------------------
// Simulation Runner
// -------------------------------------------------------------------------
pub fn run_simulation(
    reg_type: RegisterType,
    num_readers: usize,
    delay_ms: u64,
    tx: Sender<SimEvent>,
    pause_flag: Arc<AtomicBool>,
) {
    match reg_type {
        RegisterType::Safe => {
            let mut safe_reg = SafeMRSW::new(num_readers);
            let mut readers = vec![];
            for i in 0..num_readers {
                readers.push(safe_reg.get_nth_reader(i).unwrap());
            }

            let tx_writer = tx.clone();
            let writer_delay = delay_ms;
            let writer_pause = pause_flag.clone();

            thread::spawn(move || {
                let mut current_val = false;
                for _ in 1..=10 {
                    smart_sleep(0, &writer_pause);

                    current_val = !current_val;

                    // If the channel is closed (user pressed Esc), exit the thread silently
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

                    smart_sleep(writer_delay, &writer_pause);
                }
                let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
            });

            for (id, reader) in readers.into_iter().enumerate() {
                let tx_reader = tx.clone();
                let reader_delay = delay_ms;
                let reader_pause = pause_flag.clone();

                thread::spawn(move || {
                    smart_sleep(100, &reader_pause);
                    for _ in 1..=15 {
                        let value = reader.read();

                        // If the channel is closed, exit the thread silently
                        if tx_reader
                            .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                            .is_err()
                        {
                            return;
                        }

                        smart_sleep(reader_delay / 2 + 50, &reader_pause);
                    }
                });
            }
        }
        RegisterType::MRegular => {
            let mut mrsw = MRegularMRSW::new(num_readers, 11);
            let mut readers = vec![];
            for i in 0..num_readers {
                readers.push(mrsw.get_nth_reader(i).unwrap());
            }

            let tx_writer = tx.clone();
            let writer_pause = pause_flag.clone();
            thread::spawn(move || {
                for i in 1..=10 {
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

                    smart_sleep(delay_ms, &writer_pause);
                }
                let _ = tx_writer.send(SimEvent::Status("Simulation FINISHED".to_string()));
            });

            for (id, reader) in readers.into_iter().enumerate() {
                let tx_reader = tx.clone();
                let reader_pause = pause_flag.clone();
                thread::spawn(move || {
                    smart_sleep(100, &reader_pause);
                    for _ in 1..=10 {
                        let value = reader.read();

                        if tx_reader
                            .send(SimEvent::ReaderUpdate(id, format!("{}", value)))
                            .is_err()
                        {
                            return;
                        }

                        smart_sleep(delay_ms + 100, &reader_pause);
                    }
                });
            }
        }
        _ => {
            let _ = tx.send(SimEvent::Status("Pending integration.".to_string()));
        }
    }
}

// -------------------------------------------------------------------------
// Smart Sleep Function
// -------------------------------------------------------------------------
// Checks the pause flag continually so it can freeze instantly
// instead of waiting for a long sleep to expire.
fn smart_sleep(delay_ms: u64, pause_flag: &Arc<AtomicBool>) {
    let target = Duration::from_millis(delay_ms);
    let start = Instant::now();

    loop {
        // If paused, just trap the thread here checking every 50ms
        while pause_flag.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }

        // If the unpaused time has elapsed, we are done
        if start.elapsed() >= target {
            break;
        }

        // Sleep in tiny increments to remain responsive
        thread::sleep(Duration::from_millis(10));
    }
}
