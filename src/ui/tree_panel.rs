use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, AppMode, Focus, NodeKind};

pub fn draw_tree_panel(f: &mut Frame, area: Rect, app: &mut App) {
    // Render visible nodes with ASCII tree structure
    let mut items: Vec<ListItem> = Vec::new();
    for (_i, (indent, path)) in app.flat_nodes.iter().enumerate() {
        if let Some(node) = app.node_by_path(path) {
            let mut label = String::new();

            // Build ASCII tree structure
            if *indent > 0 {
                // Add tree structure for nested items
                for i in 0..*indent {
                    if i == *indent - 1 {
                        // Last connector at this depth
                        if app.is_last_child(path) {
                            label.push_str("└─ ");
                        } else {
                            label.push_str("├─ ");
                        }
                    } else {
                        // Vertical guides for ancestor levels
                        let parent_path = &path[..i + 1];
                        if app.is_last_child(parent_path) {
                            label.push_str("   ");
                        } else {
                            label.push_str("│  ");
                        }
                    }
                }
            }

            match &node.kind {
                NodeKind::Day { .. } => {
                    label.push_str(&node.label);
                }
                NodeKind::Month => {
                    let marker = if node.expanded { "[-] " } else { "[+] " };
                    label.push_str(marker);
                    label.push_str(&node.label);
                }
                NodeKind::Year => {
                    let marker = if node.expanded { "[-] " } else { "[+] " };
                    label.push_str(marker);
                    label.push_str(&node.label);
                }
            };

            items.push(ListItem::new(label));
        }
    }

    // Visual hint for focus: highlight border when Tree panel is active
    let tree_focused = matches!(app.mode, AppMode::Preview) && app.focus == Focus::Tree;
    let tree_block = Block::default()
        .title("Entries (.devlog)")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if tree_focused { Style::default().fg(Color::Yellow) } else { Style::default() });

    let list = List::new(items)
        .block(tree_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        );

    // Use ListState for highlighting current row
    let mut state = ListState::default();
    state.select(app.selected_index);
    f.render_stateful_widget(list, area, &mut state);
}
