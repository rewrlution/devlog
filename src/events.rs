use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::File;
use std::io::Read;

use crate::app::{App, AppMode, Focus};
use crate::utils::today_str;

pub fn handle_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match app.mode {
        AppMode::Preview => handle_key_preview(app, key),
        AppMode::Edit => handle_key_edit(app, key),
        AppMode::DatePrompt => handle_key_date_prompt(app, key),
        AppMode::SavePrompt => handle_key_save_prompt(app, key),
    }
}

pub fn handle_key_preview(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('n') => {
            app.date_input = today_str();
            app.date_error = None;
            app.mode = AppMode::DatePrompt;
        }
        KeyCode::Char('e') => {
            if app.current_path.is_some() {
                app.mode = AppMode::Edit;
                app.move_cursor_to_end();
            }
        }
        KeyCode::Tab => {
            // Toggle focus between tree and content in preview mode
            app.focus = if app.focus == Focus::Tree { Focus::Content } else { Focus::Tree };
        }
        KeyCode::Up => {
            if app.focus == Focus::Tree {
                app.move_selection(-1);
            } else {
                app.view_scroll = app.view_scroll.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if app.focus == Focus::Tree {
                app.move_selection(1);
            } else {
                app.view_scroll = app.view_scroll.saturating_add(1);
            }
        }
        KeyCode::Left => {
            app.toggle_expand_at_selected(false);
        }
        KeyCode::Right => {
            app.toggle_expand_at_selected(true);
        }
        KeyCode::Enter => {
            // If a file is selected, open it for viewing
            if let Some(name) = app.selected_filename().map(|s| s.to_string()) {
                let _ = app.open_file_by_name(&name);
                app.focus = Focus::Content;
            }
        }
        KeyCode::Esc => return Ok(true), // quit app from preview with Esc
        _ => {}
    }
    Ok(false)
}

pub fn handle_key_edit(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key {
        // Cross-platform save: Accept both Ctrl+S and Cmd+S
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER) => {
            app.save()?;
        }
        KeyEvent { code: KeyCode::Esc, .. } => {
            if app.dirty {
                app.mode = AppMode::SavePrompt;
                app.save_choice = 0;
            } else {
                app.mode = AppMode::Preview;
            }
        }
        KeyEvent { code: KeyCode::Left, .. } => app.move_left(),
        KeyEvent { code: KeyCode::Right, .. } => app.move_right(),
        KeyEvent { code: KeyCode::Up, .. } => app.move_up(),
        KeyEvent { code: KeyCode::Down, .. } => app.move_down(),
        KeyEvent { code: KeyCode::Backspace, .. } => app.backspace(),
        KeyEvent { code: KeyCode::Delete, .. } => app.delete(),
        KeyEvent { code: KeyCode::Enter, .. } => app.insert_newline(),
        KeyEvent { code: KeyCode::Tab, .. } => {
            // insert 2 spaces as tab
            app.insert_char(' ');
            app.insert_char(' ');
        }
        KeyEvent { code: KeyCode::Char(c), .. } => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.insert_char(c);
            }
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_key_date_prompt(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Preview;
        }
        KeyCode::Enter => {
            match App::validate_date(&app.date_input) {
                Ok(()) => {
                    app.date_error = None;
                    let date = app.date_input.clone();
                    app.open_or_create_for_date(&date)?;
                }
                Err(msg) => {
                    app.date_error = Some(msg.to_string());
                }
            }
        }
        KeyCode::Backspace => {
            app.date_input.pop();
            app.date_error = None;
        }
        KeyCode::Char(c) => {
            if c.is_ascii_digit() && app.date_input.len() < 8 {
                app.date_input.push(c);
            }
        }
        _ => {}
    }
    Ok(false)
}

pub fn handle_key_save_prompt(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Left => {
            if app.save_choice > 0 {
                app.save_choice -= 1;
            }
        }
        KeyCode::Right => {
            if app.save_choice < 2 {
                app.save_choice += 1;
            }
        }
        KeyCode::Enter => match app.save_choice {
            0 => {
                app.save()?;
                app.mode = AppMode::Preview;
            }
            1 => {
                // discard: reload from disk
                if let Some(path) = &app.current_path {
                    let mut s = String::new();
                    File::open(path)?.read_to_string(&mut s)?;
                    app.content = s;
                    app.dirty = false;
                }
                app.mode = AppMode::Preview;
            }
            _ => {
                // Cancel
                app.mode = AppMode::Edit;
            }
        },
        KeyCode::Esc => {
            app.mode = AppMode::Edit;
        }
        _ => {}
    }
    Ok(false)
}
