use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{App, AppMode};

pub mod tree_panel;
pub mod content_panel;
pub mod status_bar;
pub mod dialogs;

use tree_panel::draw_tree_panel;
use content_panel::draw_content_panel;
use status_bar::draw_status_bar;
use dialogs::{draw_date_prompt, draw_save_prompt};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Create vertical layout with status bar at bottom
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Create horizontal layout for main content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_chunks[0]);

    draw_tree_panel(f, content_chunks[0], app);
    draw_content_panel(f, content_chunks[1], app);
    draw_status_bar(f, main_chunks[1], app);

    match app.mode {
        AppMode::DatePrompt => draw_date_prompt(f, app),
        AppMode::SavePrompt => draw_save_prompt(f, app),
        _ => {}
    }
}
