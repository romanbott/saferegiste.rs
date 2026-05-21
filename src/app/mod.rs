use std::{
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
    Safe,
    Regular,
    MRegular,
    AtomicSRSW,
}

pub struct App {
    pub state: AppState,
    pub items: Vec<RegisterType>,
    pub list_state: ListState,
    pub rx: Option<Receiver<SimEvent>>,

    pub writer_logs: Vec<String>,
    pub reader_logs: Vec<Vec<String>>,
    pub status_msg: String,

    // Simulation Parameters
    pub num_readers: usize,
    pub num_reads: usize,
    pub writer_delay_ms: u64,
    pub reader_delay_ms: u64,
    pub flicker_mode: u8, // 0: Fast, 1: Normal, 2: Slow

    pub is_paused: bool,
    pub pause_flag: Option<Arc<AtomicBool>>,
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
                RegisterType::Safe,
                RegisterType::Regular,
                RegisterType::MRegular,
                RegisterType::AtomicSRSW,
            ],
            list_state,
            rx: None,
            writer_logs: Vec::new(),
            reader_logs: Vec::new(),
            status_msg: String::new(),
            num_readers: 3,
            num_reads: 100,
            writer_delay_ms: 1000,
            reader_delay_ms: 50,
            flicker_mode: 1, // Default to Normal
            is_paused: false,
            pause_flag: None,
        }
    }

    pub fn process_events(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    SimEvent::WriterUpdate(val) => {
                        self.writer_logs.push(val);
                        if self.writer_logs.len() > 100 {
                            self.writer_logs.remove(0);
                        }
                    }
                    SimEvent::ReaderUpdate(id, val) => {
                        if id < self.reader_logs.len() {
                            self.reader_logs[id].push(val);
                            if self.reader_logs[id].len() > 100 {
                                self.reader_logs[id].remove(0);
                            }
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
        self.reader_logs = vec![Vec::new(); self.num_readers];
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

        thread::spawn(move || {
            simulation::run_simulation(
                selected,
                num_readers,
                num_reads,
                writer_delay_ms,
                reader_delay_ms,
                tx,
                pause_flag,
            );
        });
    }
}
