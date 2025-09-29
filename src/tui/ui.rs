use crate::tui::models::state::{AppState, Panel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub struct UIRenderer;

impl UIRenderer {
    pub fn render(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(f.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[0]);

        Self::render_tree_panel(app_state, tree_state, f, content_chunks[0]);
        Self::render_content_panel();
        Self::render_help_footer();
    }

    fn render_tree_panel(
        app_state: &AppState,
        tree_state: &mut ListState,
        f: &mut Frame,
        area: Rect,
    ) {
        let items: Vec<ListItem> = app_state
            .flat_items
            .iter()
            .map(|(text, is_entry)| {
                // Tree art is now included in the text, no need for additional indentation
                let style = if *is_entry {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                ListItem::new(Line::from(Span::styled(text.clone(), style)))
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

    fn render_content_panel() {}

    fn render_help_footer() {}
}
