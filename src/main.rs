use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{error::Error, io, time::Duration};

mod app;

use crate::app::App;

// -------------------------------------------------------------------------
// TUI Setup & Loop
// -------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        // 1. Process any incoming messages from the simulation threads
        app.process_events();

        // 2. Draw the UI based on the current state
        terminal.draw(|f| app::ui::draw_ui(f, app))?;

        // 3. Poll for keyboard input (non-blocking, 50ms timeout to keep UI responsive)
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm::event::read()? {
                // handle_input returns true if the user pressed 'q' in the menu
                if app.handle_input(key.code) {
                    return Ok(());
                }
            }
        }
    }
}
