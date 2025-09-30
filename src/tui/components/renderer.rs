use super::{
    layout::main_layout::MainLayout,
    panels::{content_panel::ContentPanel, footer_panel::FooterPanel, tree_panel::TreePanel},
};
use crate::tui::models::state::AppState;
use ratatui::{widgets::ListState, Frame};

/// Main UI renderer that coordinates all UI components
pub struct UIRenderer;

impl UIRenderer {
    /// Renders the complete application UI by coordinating all panels and layout
    pub fn render(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame) {
        // Create the main layout areas
        let layout_areas = MainLayout::create_layout(f.area());

        // Render each panel in its designated area
        TreePanel::render(app_state, tree_state, f, layout_areas.tree_area);
        ContentPanel::render(app_state, f, layout_areas.content_area);
        FooterPanel::render(app_state, f, layout_areas.footer_area);
    }
}
