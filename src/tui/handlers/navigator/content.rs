use color_eyre::eyre::Result;
use crossterm::event::KeyCode;

use crate::tui::models::state::AppState;

pub struct ContentNavigator {}

impl ContentNavigator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_navigation(&self, key_code: KeyCode, app_state: &mut AppState) -> Result<()> {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_content_up(app_state);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_content_down(app_state);
            }
            KeyCode::Home => {
                self.reset_content_scroll(app_state);
            }
            _ => {}
        }

        Ok(())
    }

    fn scroll_content_up(&self, app_state: &mut AppState) {
        // `saturating_sub()` performs saturating subtraction, which means "saturates" at min value
        app_state.content_scroll = app_state.content_scroll.saturating_sub(1);
    }

    fn scroll_content_down(&self, app_state: &mut AppState) {
        let content_lines = app_state.selected_entry_content.lines().count();
        let max_scroll = content_lines.saturating_sub(1) as u16;
        if app_state.content_scroll < max_scroll {
            app_state.content_scroll += 1;
        }
    }

    fn reset_content_scroll(&self, app_state: &mut AppState) {
        app_state.content_scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_scroll_up() {
        let mut state = AppState::new();
        state.content_scroll = 5;
        state.scroll_content_up();
        assert_eq!(state.content_scroll, 4);

        // Test saturating behavior
        state.content_scroll = 0;
        state.scroll_content_up();
        assert_eq!(state.content_scroll, 0);
    }

    #[test]
    fn test_app_state_scroll_down() {
        let mut state = AppState::new();
        state.scroll_content_down(10);
        assert_eq!(state.content_scroll, 1);

        // Test max limit
        state.content_scroll = 10;
        state.scroll_content_down(10);
        assert_eq!(state.content_scroll, 10);
    }
}
