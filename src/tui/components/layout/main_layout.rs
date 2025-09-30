use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

/// Manages the main application layout structure
pub struct MainLayout;

impl MainLayout {
    /// Creates the main application layout with header, content, and footer areas
    pub fn create_layout(area: Rect) -> MainLayoutAreas {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(main_chunks[0]);

        MainLayoutAreas {
            tree_area: content_chunks[0],
            content_area: content_chunks[1],
            footer_area: main_chunks[1],
        }
    }
}

/// Represents the different areas of the main application layout
#[derive(Debug)]
pub struct MainLayoutAreas {
    pub tree_area: Rect,
    pub content_area: Rect,
    pub footer_area: Rect,
}