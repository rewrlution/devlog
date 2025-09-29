use crate::storage::Storage;
use crate::tui::handlers::navigation::TreeNavigator;
use crate::tui::models::state::{AppState, Panel};
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

pub struct KeyboardHandler {
    tree_navigator: TreeNavigator,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            tree_navigator: TreeNavigator::new(),
        }
    }

    pub fn handle_key_event(
        &self,
        key_code: KeyCode,
        app_state: &mut AppState,
        tree_state: &mut ListState,
    ) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => {
                app_state.should_quit = true;
            }
            KeyCode::Tab => {
                self.toggle_panel(app_state);
            }
            _ => {
                // For now, only handle tree navigation regardless of panel
                // In the future, we can add content panel navigation
                if app_state.current_panel == Panel::Nav {
                    self.tree_navigator
                        .handle_navigation(key_code, app_state, tree_state)?;
                }
            }
        }
        Ok(())
    }

    fn toggle_panel(&self, app_state: &mut AppState) {
        app_state.current_panel = match app_state.current_panel {
            Panel::Nav => Panel::Content,
            Panel::Content => Panel::Nav,
        };
    }
}
