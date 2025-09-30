use crate::tui::models::state::{AppState, Panel};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

/// Component responsible for rendering the content display panel
pub struct ContentPanel;

impl ContentPanel {
    /// Renders the content display panel
    pub fn render(app_state: &AppState, f: &mut Frame, area: Rect) {
        let content_lines: Vec<Line> = app_state
            .selected_entry_content
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();

        // Calculate scrolling - account for borders and horizontal padding
        let content_height = area.height.saturating_sub(2) as usize; // Account for borders
        let scroll_offset = app_state.content_scroll as usize;
        let visible_lines: Vec<Line> = content_lines
            .into_iter()
            .skip(scroll_offset)
            .take(content_height)
            .collect();

        let paragraph = Paragraph::new(visible_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1)) // Add horizontal padding
                    .title("Content")
                    .border_style(if app_state.current_panel == Panel::Content {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}