use crate::tui::data::{AppState, Panel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub struct UIRenderer;

impl UIRenderer {
    pub fn render(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(f.size());

        Self::render_tree_panel(app_state, tree_state, f, chunks[0]);
        Self::render_content_panel(app_state, f, chunks[1]);
    }

    fn render_tree_panel(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = app_state
            .flat_items
            .iter()
            .map(|(text, _indent, is_entry)| {
                // Tree art is now included in the text, no need for additional indentation
                let style = if *is_entry {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                ListItem::new(Line::from(Span::styled(
                    text.clone(),
                    style,
                )))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Entries")
                    .border_style(if app_state.current_panel == Panel::Tree {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .fg(Color::Black)
            );

        f.render_stateful_widget(list, area, tree_state);
    }

    fn render_content_panel(app_state: &AppState, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(app_state.selected_entry_content.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Content")
                    .border_style(if app_state.current_panel == Panel::Content {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}
