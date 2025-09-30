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
            KeyCode::End => {
                self.scroll_to_bottom(app_state);
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.scroll_content_up(app_state);
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.scroll_content_down(app_state);
                }
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

    fn scroll_to_bottom(&self, app_state: &mut AppState) {
        let content_lines = app_state.selected_entry_content.lines().count();
        let max_scroll = content_lines.saturating_sub(1) as u16;
        app_state.content_scroll = max_scroll;
    }

    fn reset_content_scroll(&self, app_state: &mut AppState) {
        app_state.content_scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_navigator_scroll_up() {
        let navigator = ContentNavigator::new();
        let mut state = AppState::new();
        state.content_scroll = 5;
        navigator.scroll_content_up(&mut state);
        assert_eq!(state.content_scroll, 4);

        // Test saturating behavior
        state.content_scroll = 0;
        navigator.scroll_content_up(&mut state);
        assert_eq!(state.content_scroll, 0);
    }

    #[test]
    fn test_content_navigator_scroll_down() {
        let navigator = ContentNavigator::new();
        let mut state = AppState::new();

        // Set up some content with multiple lines
        state.selected_entry_content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();

        navigator.scroll_content_down(&mut state);
        assert_eq!(state.content_scroll, 1);

        // Test that we can scroll to the max content
        for _ in 0..3 {
            navigator.scroll_content_down(&mut state);
        }
        assert_eq!(state.content_scroll, 4); // 5 lines, so max scroll is 4

        // Test that scrolling beyond max doesn't increase scroll
        navigator.scroll_content_down(&mut state);
        assert_eq!(state.content_scroll, 4);
    }

    #[test]
    fn test_content_navigator_reset_scroll() {
        let navigator = ContentNavigator::new();
        let mut state = AppState::new();
        state.content_scroll = 10;
        navigator.reset_content_scroll(&mut state);
        assert_eq!(state.content_scroll, 0);
    }
}
