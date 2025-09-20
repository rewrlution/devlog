use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, AppMode, Focus};

pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    // Detect platform for key binding display
    let save_key = if cfg!(target_os = "macos") {
        "Cmd+S"
    } else {
        "Ctrl+S"
    };
    
    let status_text = match app.mode {
        AppMode::Preview => {
            let focus_str = match app.focus { Focus::Tree => "Tree", Focus::Content => "Content" };
            let arrows_hint = match app.focus {
                Focus::Tree => "↑↓: Navigate Tree | ←→: Collapse/Expand",
                Focus::Content => "↑↓: Scroll Content",
            };
            format!(
                "VIEW MODE | Focus: {} | {} | Enter: Open | e: Edit | n: New | Tab: Switch Focus | Esc: Quit",
                focus_str,
                arrows_hint,
            )
        }
        AppMode::Edit => {
            format!(
                "EDIT MODE | Focus: Content | Esc: Back to View | {}: Save | Arrow keys: Move cursor",
                save_key
            )
        }
        AppMode::DatePrompt => {
            "NEW ENTRY | Enter date (YYYYMMDD) | Enter: Create | Esc: Cancel".to_string()
        }
        AppMode::SavePrompt => {
            "SAVE CHANGES | ←→: Select option | Enter: Confirm | Esc: Cancel".to_string()
        }
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Help")
        )
        .wrap(Wrap { trim: false });
    
    f.render_widget(status_paragraph, area);
}
