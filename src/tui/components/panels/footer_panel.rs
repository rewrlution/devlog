use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::models::state::{AppState, Panel};

/// Component responsible for rendering the help footer panel
pub struct FooterPanel;

impl FooterPanel {
    /// Renders the help footer panel with keyboard shortcuts
    pub fn render(app_state: &AppState, f: &mut Frame, area: Rect) {
        let help_text_nav = vec![Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch Panel | "),
            Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
            Span::raw(": Move | "),
            Span::styled("→/l/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Expand | "),
            Span::styled("←/h/Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Collapse | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ])];

        let help_text_content = vec![Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch Panel | "),
            Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
            Span::raw(": Scroll | "),
            Span::styled("Home", Style::default().fg(Color::Yellow)),
            Span::raw(": Top | "),
            Span::styled("End", Style::default().fg(Color::Yellow)),
            Span::raw(": Bottom | "),
            Span::styled("PageUp", Style::default().fg(Color::Yellow)),
            Span::raw(": Page Up | "),
            Span::styled("PageDown", Style::default().fg(Color::Yellow)),
            Span::raw(": Page Down | "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ])];

        let help_text = match app_state.current_panel {
            Panel::Nav => help_text_nav,
            Panel::Content => help_text_content,
        };

        let help_paragraph = Paragraph::new(help_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(Color::Gray)),
        );

        f.render_widget(help_paragraph, area);
    }
}
