use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, prelude::Backend, widgets::ListState};

use crate::app::simulation::run_simulation;

mod simulation;
mod ui;

use ui::draw_ui;

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
    state: AppState,
    items: Vec<RegisterType>,
    list_state: ListState,
    rx: Option<Receiver<SimEvent>>,

    writer_logs: Vec<String>,
    reader_logs: Vec<Vec<String>>,
    status_msg: String,

    num_readers: usize,
    delay_ms: u64,

    // Pause state tracking
    is_paused: bool,
    pause_flag: Option<Arc<AtomicBool>>,
}

impl App {
    pub fn new() -> App {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
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
            delay_ms: 500,
            is_paused: false,
            pause_flag: None,
        }
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
        let delay = self.delay_ms;

        thread::spawn(move || {
            run_simulation(selected, num_readers, delay, tx, pause_flag);
        });
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>>
where
    <B as Backend>::Error: 'static,
{
    loop {
        if let Some(rx) = &app.rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    SimEvent::WriterUpdate(val) => {
                        app.writer_logs.push(val);
                        if app.writer_logs.len() > 100 {
                            app.writer_logs.remove(0);
                        }
                    }
                    SimEvent::ReaderUpdate(id, val) => {
                        if id < app.reader_logs.len() {
                            app.reader_logs[id].push(val);
                            if app.reader_logs[id].len() > 100 {
                                app.reader_logs[id].remove(0);
                            }
                        }
                    }
                    SimEvent::Status(msg) => app.status_msg = msg,
                }
            }
        }

        terminal.draw(|f| draw_ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Menu => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => app.start_simulation(),
                        KeyCode::Right => app.num_readers += 1,
                        KeyCode::Left => {
                            if app.num_readers > 1 {
                                app.num_readers -= 1;
                            }
                        }
                        _ => {}
                    },
                    AppState::Running => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.state = AppState::Menu;
                            app.rx = None;

                            // Unpause strictly to allow threads to close out naturally
                            // if they are sleeping (prevents orphaned paused threads)
                            if let Some(flag) = &app.pause_flag {
                                flag.store(false, Ordering::SeqCst);
                            }
                        }
                        // Handle Pause/Play Hotkeys
                        KeyCode::Char('p') | KeyCode::Char(' ') => {
                            app.is_paused = !app.is_paused;
                            if let Some(flag) = &app.pause_flag {
                                flag.store(app.is_paused, Ordering::SeqCst);
                            }

                            if !app.status_msg.contains("FINISHED") {
                                app.status_msg = if app.is_paused {
                                    "Simulation PAUSED".to_string()
                                } else {
                                    "Simulation RUNNING".to_string()
                                };
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
