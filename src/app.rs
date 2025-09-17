use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent};

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current application mode
    pub mode: AppMode,
    /// Whether the application should quit
    pub should_quit: bool,
}

/// Application mode
#[derive(Debug, PartialEq, Clone)]
pub enum AppMode {
    /// Navigating through entries
    Navigation,
    /// Edit an entry
    Edit,
    /// Showing a prompt
    Prompt(PromptType),
}

/// Prompt types
#[derive(Debug, PartialEq, Clone)]
pub enum PromptType {
    CreateEntry,
    DeleteConfirmation,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            mode: AppMode::Navigation,
            should_quit: false,
        }
    }

    /// Handle a key event based on current mode
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Navigation => self.handle_navigation_key(key),
            // Future modes will go here
            _ => Ok(()),
        }
    }

    /// Handel keys when in vaigation mode
    fn handle_navigation_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            // Future navigation keys here
            _ => {}
        }
        Ok(())
    }

    /// Check if the app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get current mode as a string for display
    pub fn mode_string(&self) -> &'static str {
        match self.mode {
            AppMode::Navigation => "Navigation",
            AppMode::Edit => "Edit",
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_new_app() {
        let app = App::new();
        assert_eq!(app.mode, AppMode::Navigation);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_quit_key() {
        let mut app = App::new();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);

        app.handle_key(key).unwrap();
        assert!(app.should_quit());
    }

    #[test]
    fn test_mode_string() {
        let app = App::new();
        assert_eq!(app.mode_string(), "Navigation");
    }
}
