use crate::tui::models::state::AppState;
use crate::tui::tree::flattener::FlatTreeItem;
use crate::{storage::Storage, utils::editor};
use color_eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::widgets::ListState;
use std::io;

pub struct EditorHandler {
    storage: Storage,
}

impl EditorHandler {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn editor_current_entry(
        &self,
        app_state: &mut AppState,
        tree_state: &ListState,
        flat_items: &[FlatTreeItem],
    ) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if let Some((text, is_entry)) = flat_items.get(selected) {
                if *is_entry {
                    // Extract entry ID from display text. Examples are:
                    // "└─ 20250920" -> "20250920"
                    // "├─ 20241231" -> "20241231"
                    // "│   └─ 20250920" -> "20250920"
                    let entry_id = &text[text.len() - 8..];
                    self.launch_editor_for_entry(&entry_id, app_state)?;
                }
            }
        }
        Ok(())
    }

    fn launch_editor_for_entry(&self, entry_id: &str, app_state: &mut AppState) -> Result<()> {
        // Save current terminal state and exit TUI mode
        self.exit_tui_mode()?;

        let result = self.edit_entry_content(entry_id);

        // Restore TUI mode
        self.enter_tui_mode()?;

        // Handle editor result
        match result {
            Ok(_) => {
                // Refresh the content in the TUI by reloading the entry
                if let Ok(entry) = self.storage.load_entry(entry_id) {
                    app_state.selected_entry_content = entry.content;
                    app_state.reset_content_scroll();
                }
                app_state.needs_redraw = true;
            }
            Err(e) => return Err(e),
        }

        Ok(())
    }

    fn edit_entry_content(&self, entry_id: &str) -> Result<()> {
        let mut entry = self.storage.load_entry(entry_id)?;
        let new_content = editor::launch_editor(Some(&entry.content))?;
        entry.update_content(new_content);
        self.storage.save_entry(&entry)
    }

    /// Save current terminal state and exit TUI mode
    fn exit_tui_mode(&self) -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Restore TUI mode with proper screen clearing
    fn enter_tui_mode(&self) -> Result<()> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
}
