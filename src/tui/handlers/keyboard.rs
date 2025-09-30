use crate::storage::{self, Storage};
use crate::tui::handlers::navigator::content::ContentNavigator;
use crate::tui::handlers::navigator::tree::TreeNavigator;
use crate::tui::models::state::{AppState, Panel};
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

pub struct KeyboardHandler {
    tree_navigator: TreeNavigator,
    content_navigator: ContentNavigator,
}

impl KeyboardHandler {
    pub fn new(storage: Storage) -> Self {
        Self {
            tree_navigator: TreeNavigator::new(storage),
            content_navigator: ContentNavigator::new(),
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
            _ => match app_state.current_panel {
                Panel::Nav => {
                    self.tree_navigator
                        .handle_navigation(key_code, app_state, tree_state)?;
                }
                Panel::Content => {
                    self.content_navigator
                        .handle_navigation(key_code, app_state)?;
                }
            },
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
