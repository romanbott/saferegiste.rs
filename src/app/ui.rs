// src/ui.rs

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, AppState, RegisterType};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    match app.state {
        AppState::Menu => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                // Increased to Length(8) to fit the Flicker Mode text
                .constraints([Constraint::Length(8), Constraint::Min(0)].as_ref())
                .split(f.size());

            let mode_str = match app.flicker_mode {
                0 => "Fast",
                1 => "Normal",
                _ => "Slow",
            };

            let menu_text = format!(
                "Select Register | Enter to Start | 'q' to quit\n\
                 Readers       (Left/Right): {}\n\
                 Reads per run   ('n'/'N') : {}\n\
                 Writer Delay    ('w'/'W') : {}ms\n\
                 Reader Delay    ('r'/'R') : {}ms\n\
                 Flicker Mode      ('f')   : {}",
                app.num_readers, app.num_reads, app.writer_delay_ms, app.reader_delay_ms, mode_str
            );

            let title = Paragraph::new(menu_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Configuration"),
            );
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
                        Constraint::Length(3),
                        Constraint::Length(10),
                        Constraint::Min(0),
                    ]
                    .as_ref(),
                )
                .split(f.size());

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
            let writer_list = List::new(writer_items)
                .block(Block::default().borders(Borders::ALL).title("Writer"));
            f.render_widget(writer_list, main_chunks[1]);

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
                        .title(format!("Reader {}", i)),
                );

                f.render_widget(reader_list, reader_chunks[i]);
            }
        }
    }
}
