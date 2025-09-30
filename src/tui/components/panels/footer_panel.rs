use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Component responsible for rendering the help footer panel
pub struct FooterPanel;

impl FooterPanel {
    /// Renders the help footer panel with keyboard shortcuts
    pub fn render(f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(": Switch Panel | "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate | "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": Select | "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(": Quit"),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .border_style(Style::default().fg(Color::Gray)),
            );

        f.render_widget(help_paragraph, area);
    }
}