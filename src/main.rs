use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    error::Error,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

mod atomic_mrsw;
mod atomic_srsw;
mod m_regular;
mod regular_registers;
mod safe_mrsw;
mod safe_registers;
mod stamped_values;

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

struct App {
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
    fn new() -> App {
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

// -------------------------------------------------------------------------
// Simulation Runner
// -------------------------------------------------------------------------
// -------------------------------------------------------------------------
// Simulation Runner
// -------------------------------------------------------------------------
fn run_simulation(
    reg_type: RegisterType,
    num_readers: usize,
    delay_ms: u64,
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
            let mut mrsw = m_regular::MRegularMRSW::new(num_readers, 11);
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
// TUI Setup & Loop
// -------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>>
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

        terminal.draw(|f| ui(f, &mut app))?;

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

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    match app.state {
        AppState::Menu => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let title = Paragraph::new(format!(
                "Select Register | Readers: {} (Left/Right to change) | 'q' to quit",
                app.num_readers
            ))
            .block(Block::default().borders(Borders::ALL).title("Menu"));
            f.render_widget(title, chunks[0]);

            let items: Vec<ListItem> = app
                .items
                .iter()
                .map(|i| {
                    let name = match i {
                        RegisterType::Safe => "Safe Boolean SRSW/MRSW",
                        RegisterType::Regular => "Regular MRSW",
                        RegisterType::MRegular => "M-Valued Regular MRSW",
                        RegisterType::AtomicSRSW => "Atomic SRSW",
                    };
                    ListItem::new(Line::from(Span::raw(name)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Registers"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);
        }
        AppState::Running => {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),  // Status
                        Constraint::Length(10), // Writer Pane
                        Constraint::Min(0),     // Readers Panes Area
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // 1. Status Bar (Dynamic color based on pause state)
            let status_style = if app.is_paused {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            let status = Paragraph::new(format!(
                "{} | Press 'p' or Space to Pause/Play | 'q' or 'Esc' to return",
                app.status_msg
            ))
            .style(status_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Simulation Status"),
            );
            f.render_widget(status, main_chunks[0]);

            // 2. Writer Pane
            let writer_items: Vec<ListItem> = app
                .writer_logs
                .iter()
                .rev()
                .map(|log| {
                    ListItem::new(Line::from(Span::styled(
                        log.clone(),
                        Style::default().fg(Color::Yellow),
                    )))
                })
                .collect();
            let writer_list = List::new(writer_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Writer (Newest on top)"),
            );
            f.render_widget(writer_list, main_chunks[1]);

            // 3. Reader Panes
            let reader_constraints: Vec<Constraint> = (0..app.num_readers)
                .map(|_| Constraint::Ratio(1, app.num_readers as u32))
                .collect();

            let reader_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(reader_constraints)
                .split(main_chunks[2]);

            for (i, logs) in app.reader_logs.iter().enumerate() {
                let reader_items: Vec<ListItem> = logs
                    .iter()
                    .rev()
                    .map(|log| {
                        ListItem::new(Line::from(Span::styled(
                            log.clone(),
                            Style::default().fg(Color::Cyan),
                        )))
                    })
                    .collect();

                let reader_list = List::new(reader_items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Reader {} (Newest on top)", i)),
                );

                f.render_widget(reader_list, reader_chunks[i]);
            }
        }
    }
}
