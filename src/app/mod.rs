use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
};

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;
use registers::safe_registers::FLICKER_MODE;

mod simulation;
pub mod ui;

enum SimEvent {
    WriterUpdate(String),
    ReaderUpdate(usize, String),
    Status(String),
}

enum AppState {
    Menu,
    Running,
}

#[derive(Clone, Copy)]
enum RegisterType {
    SafeSRSW,
    SafeMRSW,
    Regular,
    MRegular,
    AtomicSRSW,
    AtomicMRSW,
    AtomicMRMW,
}

impl RegisterType {
    fn is_boolean(&self) -> bool {
        match self {
            RegisterType::MRegular => false,
            RegisterType::AtomicSRSW => false,
            _ => true,
        }
    }
}

const LOG_LEN: usize = 5000;

const NUM_SEQS: usize = 3;

pub const BOOLEAN_SEQUENCES: &[&[bool]] = &[
    &[true, false, true, false, true, false, true, false], // Alternating
    &[true, true, false, false, true, true, false, false], // Paired
    &[true, true, true, true, false, false, false, false], // Blocks
];

pub const NUMERIC_SEQUENCES: &[&[u8]] = &[
    &[0, 1, 2, 3, 4, 5, 6, 7], // Alternating
    &[0, 0, 2, 2, 4, 4, 6, 6], // Paired
    &[0, 0, 0, 0, 1, 1, 1, 1], // Blocks
];

pub struct App {
    pub state: AppState,
    pub items: Vec<RegisterType>,
    pub list_state: ListState,
    pub rx: Option<Receiver<SimEvent>>,

    pub writer_logs: VecDeque<String>,
    pub reader_logs: Vec<VecDeque<String>>,
    pub status_msg: String,

    // Simulation Parameters
    pub num_readers: usize,
    pub num_reads: usize,
    pub writer_delay_ms: u64,
    pub reader_delay_ms: u64,
    pub flicker_mode: u8, // 0: Fast, 1: Normal, 2: Slow
    pub sequence_idx: usize,

    pub is_paused: bool,
    pub pause_flag: Option<Arc<AtomicBool>>,
    pub scroll_offset: usize,
}

impl App {
    pub fn new() -> App {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        // Ensure the global config matches our default state
        FLICKER_MODE.store(1, Ordering::Relaxed);

        App {
            state: AppState::Menu,
            items: vec![
                RegisterType::SafeSRSW,
                RegisterType::SafeMRSW,
                RegisterType::Regular,
                RegisterType::MRegular,
                RegisterType::AtomicSRSW,
                RegisterType::AtomicMRSW,
                RegisterType::AtomicMRMW,
            ],
            list_state,
            rx: None,
            writer_logs: VecDeque::with_capacity(LOG_LEN),
            reader_logs: Vec::new(),
            status_msg: String::new(),
            num_readers: 3,
            num_reads: 100,
            writer_delay_ms: 1000,
            reader_delay_ms: 50,
            flicker_mode: 1, // Default to Normal
            is_paused: false,
            pause_flag: None,
            scroll_offset: 0,
            sequence_idx: 0,
        }
    }

    pub fn process_events(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    SimEvent::WriterUpdate(val) => {
                        if self.writer_logs.len() >= LOG_LEN {
                            self.writer_logs.pop_front();
                        }
                        self.writer_logs.push_back(val);
                    }
                    SimEvent::ReaderUpdate(id, val) => {
                        if id < self.reader_logs.len() {
                            if self.reader_logs[id].len() >= LOG_LEN {
                                self.reader_logs[id].pop_front();
                            }
                            self.reader_logs[id].push_back(val);
                        }
                    }
                    SimEvent::Status(msg) => self.status_msg = msg,
                }
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match self.state {
            AppState::Menu => match key {
                KeyCode::Char('q') => return true,
                KeyCode::Down | KeyCode::Char('j') => self.next(),
                KeyCode::Up | KeyCode::Char('k') => self.previous(),
                KeyCode::Enter => self.start_simulation(),

                // Hotkeys for Parameters
                KeyCode::Right => self.num_readers += 1,
                KeyCode::Left => {
                    if self.num_readers > 1 {
                        self.num_readers -= 1;
                    }
                }
                KeyCode::Char('n') => {
                    if self.num_reads > 1 {
                        self.num_reads /= 2;
                    }
                }
                KeyCode::Char('N') => self.num_reads *= 2,
                KeyCode::Char('w') => {
                    if self.writer_delay_ms > 100 {
                        self.writer_delay_ms -= 100;
                    }
                }
                KeyCode::Char('W') => self.writer_delay_ms += 100,
                KeyCode::Char('r') => {
                    if self.reader_delay_ms > 1 {
                        self.reader_delay_ms = (self.reader_delay_ms * 9) / 10;
                    }
                }
                KeyCode::Char('R') => self.reader_delay_ms += 100,

                // Hotkey for Flicker Mode
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    self.flicker_mode = (self.flicker_mode + 1) % 3;
                    // Update global config instantly
                    FLICKER_MODE.store(self.flicker_mode, Ordering::Relaxed);
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.sequence_idx = (self.sequence_idx + 1) % NUM_SEQS;
                }

                _ => {}
            },
            AppState::Running => match key {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.state = AppState::Menu;
                    self.rx = None;
                    if let Some(flag) = &self.pause_flag {
                        flag.store(false, Ordering::SeqCst);
                    }
                }
                KeyCode::Char('p') | KeyCode::Char(' ') => {
                    self.is_paused = !self.is_paused;
                    if !self.is_paused {
                        self.scroll_offset = 0;
                    }

                    if let Some(flag) = &self.pause_flag {
                        flag.store(self.is_paused, Ordering::SeqCst);
                    }
                    if !self.status_msg.contains("FINISHED") {
                        self.status_msg = if self.is_paused {
                            "Simulation PAUSED".to_string()
                        } else {
                            "Simulation RUNNING".to_string()
                        };
                    }
                }
                KeyCode::Down => {
                    if self.is_paused {
                        // Find the maximum length across all logs to prevent over-scrolling
                        let max_reader_len =
                            self.reader_logs.iter().map(|l| l.len()).max().unwrap_or(0);
                        let absolute_max = max_reader_len.max(self.writer_logs.len());

                        if self.scroll_offset < absolute_max.saturating_sub(1) {
                            self.scroll_offset += 1;
                        }
                    }
                }
                KeyCode::Up => {
                    if self.is_paused {
                        self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    }
                }

                _ => {}
            },
        }
        false
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn start_simulation(&mut self) {
        self.state = AppState::Running;
        self.writer_logs.clear();
        self.reader_logs = vec![VecDeque::with_capacity(LOG_LEN); self.num_readers];
        self.status_msg = String::from("Simulation RUNNING");
        self.is_paused = false;

        let pause_flag = Arc::new(AtomicBool::new(false));
        self.pause_flag = Some(pause_flag.clone());

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let selected = self.items[self.list_state.selected().unwrap()];
        let num_readers = self.num_readers;
        let num_reads = self.num_reads;
        let writer_delay_ms = self.writer_delay_ms;
        let reader_delay_ms = self.reader_delay_ms;

        let seq_idx = self.sequence_idx;

        thread::spawn(move || {
            simulation::run_simulation(
                selected,
                num_readers,
                num_reads,
                writer_delay_ms,
                reader_delay_ms,
                seq_idx,
                tx,
                pause_flag,
            );
        });
    }
}
