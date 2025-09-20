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
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(f.size());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[0]);

        Self::render_tree_panel(app_state, tree_state, f, content_chunks[0]);
        Self::render_content_panel(app_state, f, content_chunks[1]);
        Self::render_help_footer(app_state, f, main_chunks[1]);
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
        let content_lines: Vec<Line> = app_state.selected_entry_content
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();

        // Calculate scrolling - account for borders and horizontal padding
        let content_height = area.height.saturating_sub(2) as usize; // Account for borders
        let total_lines = content_lines.len();
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
                    .padding(ratatui::widgets::Padding::horizontal(1)) // Add horizontal padding
                    .title(if total_lines > content_height {
                        format!("Content ({}/{} lines)", 
                            (scroll_offset + content_height).min(total_lines),
                            total_lines
                        )
                    } else {
                        "Content".to_string()
                    })
                    .border_style(if app_state.current_panel == Panel::Content {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn render_help_footer(app_state: &AppState, f: &mut Frame, area: Rect) {
        let help_text = match app_state.current_panel {
            Panel::Tree => "Navigation: ↑↓/jk=move, →/l/Enter=expand, ←/h=collapse, Tab=switch panel, q=quit",
            Panel::Content => "Navigation: ↑↓/jk=scroll, Tab=switch panel, e=edit entry, q=quit",
        };

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, area);
    }
}
