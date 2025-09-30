use crate::tui::models::state::{AppState, Panel};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Component responsible for rendering the navigation tree panel
pub struct TreePanel;

impl TreePanel {
    /// Renders the tree navigation panel
    pub fn render(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = app_state
            .flat_items
            .iter()
            .map(|(_, display_text, is_entry)| {
                let style = if *is_entry {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                ListItem::new(Line::from(Span::styled(display_text.clone(), style)))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Entries")
                    .border_style(if app_state.current_panel == Panel::Nav {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black));

        f.render_stateful_widget(list, area, tree_state);
    }
}
