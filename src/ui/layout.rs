use std::default;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
/// Render the main UI layout
pub fn render(app: &App, frame: &mut Frame) {
    // Split screen: main area + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Main area (at least 3 lines)
            Constraint::Length(1), // Status bar (exactly 1 line)
        ])
        .split(frame.area());

    // Render main content area
    render_main_content(app, frame, chunks[0]);

    // Render status bar
    render_status_bar(app, frame, chunks[1]);
}

/// Render the main content area
fn render_main_content(app: &App, frame: &mut Frame, area: Rect) {
    let content = format!(
        "DevLog\n\nCurrent Mode: {}\n\nPress 'q' to quit",
        app.mode_string()
    );

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("DevLog v1.0.0")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

/// Render the status bar at the bottom
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let status_text = match app.mode {
        crate::app::AppMode::Navigation => "[q] Quit | Mode: Navigation",
        _ => "",
    };

    let paragraph = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(Block::default());

    frame.render_widget(paragraph, area);
}
