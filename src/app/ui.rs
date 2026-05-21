use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, AppState, RegisterType};

pub fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
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
