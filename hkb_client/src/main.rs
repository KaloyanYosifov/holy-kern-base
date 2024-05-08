use app_state::AppView;
use components::{Component, Navigation};
use crossterm::event::{self, Event, KeyCode};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::logger::init as logger_init;
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use std::{io::Error as IOError, thread, time::Duration};
use thiserror::Error as ThisError;

mod app_state;
mod apps;
mod components;
mod events;
mod focus;
mod terminal;
mod utils;

pub const APP_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

#[derive(ThisError, Debug)]
pub enum RendererError {
    #[error("Failed to render output!")]
    FailedToRenderToOutput(#[from] IOError),
    #[error("Failed to initialize terminal")]
    FailedToInitializeTerminal(#[from] terminal::TerminalError),
}

type RenderResult = Result<(), RendererError>;

fn main() -> RenderResult {
    let mut terminal = terminal::init()?;
    let mut should_quit = false;
    let mut main_app = apps::MainApp::new();
    let mut reminders_app = apps::RemindersApp::new();
    let mut navigation =
        Navigation::new("HKB".to_string(), vec![AppView::Main, AppView::Reminders]);

    logger_init(None);
    terminal.clear()?;

    // TODO: do not use in memory sqlite database here
    let database_file_path = dirs::data_local_dir().unwrap().join("hkb/db");
    init_database(
        database_file_path.to_str().unwrap(),
        vec![CORE_MIGRATIONS, APP_MIGRATIONS],
    )
    .expect("Failed to initialize database!");

    while !should_quit {
        while event::poll(Duration::ZERO).unwrap() {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(event) => match event.code {
                        KeyCode::Char(c) => {
                            should_quit =
                                c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL)
                        }
                        KeyCode::Esc => app_state::set_editing(false),
                        _ => {}
                    },
                    _ => {}
                }

                events::push(event);
            }
        }

        terminal.draw(|frame| {
            let base_layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ],
            )
            .split(frame.size());
            navigation.render(frame, base_layout[0]);
            frame.render_widget(
                Block::new()
                    .borders(Borders::TOP)
                    .title(if app_state::is_editing() {
                        "Insert mode"
                    } else {
                        "Normal Mode"
                    }),
                base_layout[2],
            );

            match app_state::get_view() {
                AppView::Main => main_app.render(frame, base_layout[1]),
                AppView::Reminders => reminders_app.render(frame, base_layout[1]),
            };
        })?;

        events::clear();

        // 60 FPS = 16 millis. Since poll is blocking we can simulate it as a sleep
        thread::sleep(Duration::from_millis(16));
    }

    terminal::close()?;

    Ok(())
}
