use crate::tui::models::state::{AppState, Panel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Wrap},
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
        Self::render_content_panel(app_state, f, content_chunks[1]);
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

    fn render_content_panel(app_state: &AppState, f: &mut Frame, area: Rect) {
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

        // Above - line content calculation logic

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

    fn render_help_footer() {}
}
