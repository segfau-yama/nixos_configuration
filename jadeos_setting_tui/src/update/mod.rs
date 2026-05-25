use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::{App, InputMode, Screen};

pub(crate) mod pc_config;
pub(crate) mod user_flow;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind == KeyEventKind::Release {
        return;
    }

    match key.code {
        KeyCode::Char('q') if !app.active_field_accepts_text() => app.should_quit = true,
        KeyCode::Right => app.confirm_current_screen(),
        KeyCode::Enter => {
            if app.active_field_accepts_text() {
                if app.is_editing() {
                    app.confirm_current_screen();
                } else {
                    app.set_input_mode(InputMode::Editing);
                }
            } else {
                app.confirm_current_screen();
            }
        }
        KeyCode::Left => app.previous_screen_for_current_flow(),
        KeyCode::Up => app.move_active_field(-1),
        KeyCode::Down | KeyCode::Tab => app.move_active_field(1),
        KeyCode::Backspace => {
            if app.active_field_accepts_text() {
                if app.is_editing() {
                    app.backspace_active_field();
                } else {
                    app.set_input_mode(InputMode::Editing);
                }
            }
        }
        KeyCode::Char(' ') => {
            if app.active_field_accepts_text() && app.is_editing() {
                app.insert_active_field_char(' ');
            } else {
                app.toggle_active_field();
            }
        }
        KeyCode::Char(c) => {
            if app.active_field_accepts_text() {
                if !app.is_editing() {
                    app.set_input_mode(InputMode::Editing);
                }
                app.insert_active_field_char(c);
            }
        }
        KeyCode::Esc => {
            if app.is_editing() {
                app.set_input_mode(InputMode::Normal);
            } else if app.screen == Screen::Welcome {
                app.should_quit = true;
            } else {
                app.previous_screen_for_current_flow();
            }
        }
        _ => {}
    }
}
