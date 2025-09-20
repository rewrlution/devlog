use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

use crate::app::{App, AppMode, Focus, NodeKind};
use crate::markdown::render_markdown_simple;

pub fn ui(f: &mut Frame, app: &mut App) {
    // Create vertical layout with status bar at bottom
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    // Create horizontal layout for main content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_chunks[0]);

    draw_left(f, content_chunks[0], app);
    draw_right(f, content_chunks[1], app);
    draw_status_bar(f, main_chunks[1], app);

    match app.mode {
        AppMode::DatePrompt => draw_date_prompt(f, app),
        AppMode::SavePrompt => draw_save_prompt(f, app),
        _ => {}
    }
}

pub fn draw_left(f: &mut Frame, area: Rect, app: &mut App) {
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
        .border_style(if tree_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let list = List::new(items).block(tree_block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    // Use ListState for highlighting current row
    let mut state = ListState::default();
    state.select(app.selected_index);
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_right(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match (&app.current_path, app.mode) {
        (Some(p), AppMode::Edit) => format!(
            "EDIT — {}{}",
            p.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            if app.dirty { " — ●" } else { "" }
        ),
        (Some(p), AppMode::Preview) => format!(
            "VIEW — {}",
            p.file_name().and_then(|s| s.to_str()).unwrap_or("")
        ),
        (Some(p), _) => p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string(),
        (None, _) => "No entry".to_string(),
    };

    // Compute inner drawing area first so we can pre-wrap text by character width
    let inner_x = area.x.saturating_add(1);
    let inner_y = area.y.saturating_add(1);
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);

    // Reserve space differently for Preview vs Edit
    let (line_num_width, content_w): (u16, u16) = if matches!(app.mode, AppMode::Edit) {
        // Calculate line number width (minimum 3 characters for line numbers)
        let total_lines = app.content.lines().count();
        let lnw = (total_lines.to_string().len().max(3) + 1) as u16; // +1 for space after number
        (lnw, inner_w.saturating_sub(lnw + 1)) // +1 for scrollbar
    } else {
        // Preview mode: no line numbers. Reserve 1 col for scrollbar only.
        (0, inner_w.saturating_sub(1))
    };

    // Build display lines based on mode (Preview renders Markdown, Edit shows with line numbers)
    let text: Vec<Line> = if app.files.is_empty() && app.current_path.is_none() {
        vec![
            Line::from("No entries."),
            Line::from("Press n to create today's entry."),
        ]
    } else {
        if matches!(app.mode, AppMode::Preview) {
            render_markdown_simple(&app.content, content_w as usize)
        } else {
            // Edit mode with line numbers
            let mut out: Vec<Line> = Vec::new();
            let width = content_w as usize;
            let content_lines: Vec<&str> = app.content.split('\n').collect();
            let line_num_style = Style::default().fg(Color::DarkGray);
            let line_num_width_usize = line_num_width.saturating_sub(1) as usize;
            for (line_idx, raw_line) in content_lines.iter().enumerate() {
                let line_num = line_idx + 1;
                let line_num_str = format!("{:>width$} ", line_num, width = line_num_width_usize);
                if width == 0 {
                    out.push(Line::from(vec![
                        Span::styled(line_num_str, line_num_style),
                        Span::raw(*raw_line),
                    ]));
                    continue;
                }
                // Handle line wrapping with line numbers
                let mut buf = String::new();
                let mut count = 0usize;
                let mut is_first_segment = true;
                for ch in raw_line.chars() {
                    buf.push(ch);
                    count += 1;
                    if count == width {
                        let line_prefix = if is_first_segment {
                            Span::styled(line_num_str.clone(), line_num_style)
                        } else {
                            Span::styled(
                                format!("{:>width$} ", "", width = line_num_width_usize),
                                line_num_style,
                            )
                        };
                        out.push(Line::from(vec![line_prefix, Span::raw(buf.clone())]));
                        buf.clear();
                        count = 0;
                        is_first_segment = false;
                    }
                }
                let line_prefix = if is_first_segment {
                    Span::styled(line_num_str, line_num_style)
                } else {
                    Span::styled(
                        format!("{:>width$} ", "", width = line_num_width_usize),
                        line_num_style,
                    )
                };
                if !buf.is_empty() {
                    out.push(Line::from(vec![line_prefix, Span::raw(buf)]));
                } else if raw_line.is_empty() || is_first_segment {
                    out.push(Line::from(vec![line_prefix, Span::raw("")]));
                }
            }
            out
        }
    };

    // Apply vertical scrolling based on view_scroll and inner height
    let max_start = text.len().saturating_sub(1);
    let mut start = app.view_scroll.min(max_start);
    let height = inner_h as usize;
    // Ensure start is not beyond what would show empty space unless content shorter than height
    if height > 0 {
        let max_valid_start = text.len().saturating_sub(height).min(max_start);
        start = start.min(max_valid_start);
    }
    // Persist clamped scroll so external key handlers don't accumulate past-end values
    app.view_scroll = start;
    let end = if height == 0 {
        start
    } else {
        (start + height).min(text.len())
    };
    let visible: Vec<Line> = if start < end {
        text[start..end].to_vec()
    } else {
        Vec::new()
    };

    // Visual hint for focus: highlight border when Content panel is active (Preview+Content or Edit mode)
    let content_focused = matches!(app.mode, AppMode::Edit)
        || (matches!(app.mode, AppMode::Preview) && app.focus == Focus::Content);
    let right_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if content_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let paragraph = Paragraph::new(visible).block(right_block);
    f.render_widget(paragraph, area);

    // Draw vertical scrollbar in reserved 1-column strip when there is overflow
    if content_w > 0 {
        let total = text.len();
        let view_h = inner_h as usize;
        if total > view_h && view_h > 0 {
            // Position scrollbar at the very right edge (after any line numbers + content)
            let bar_x = inner_x + line_num_width + content_w;
            let bar_area = Rect {
                x: bar_x,
                y: inner_y,
                width: 1,
                height: inner_h,
            };

            // Track uses interior space between caps (top/bottom). If height is too small, fall back.
            let has_caps = view_h >= 3;
            let track_h = if has_caps {
                view_h.saturating_sub(2)
            } else {
                view_h
            };

            // Compute thumb size and position within the track
            let thumb_h = ((track_h * track_h) / total).max(1).min(track_h.max(1));
            let max_top = track_h.saturating_sub(thumb_h);
            let denom = total.saturating_sub(view_h).max(1);
            let thumb_top = if total > view_h {
                (start * max_top) / denom
            } else {
                0
            };

            // Build scrollbar glyphs line-by-line
            let mut lines: Vec<Line> = Vec::with_capacity(view_h);
            for i in 0..view_h {
                if has_caps && i == 0 {
                    lines.push(Line::from(Span::styled(
                        "▲",
                        Style::default().fg(Color::DarkGray),
                    )));
                    continue;
                }
                if has_caps && i == view_h - 1 {
                    lines.push(Line::from(Span::styled(
                        "▼",
                        Style::default().fg(Color::DarkGray),
                    )));
                    continue;
                }

                let track_i = if has_caps { i - 1 } else { i };
                if track_i >= thumb_top && track_i < thumb_top + thumb_h {
                    lines.push(Line::from(Span::styled(
                        "█",
                        Style::default().fg(Color::Gray),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        "│",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            let sb = Paragraph::new(lines);
            f.render_widget(sb, bar_area);
        }
    }

    // Show the text cursor in edit mode, using the same pre-wrapping model
    if matches!(app.mode, AppMode::Edit) {
        // visual_row is the number of wrapped segments before the cursor row,
        // plus additional wraps within the current row up to cursor_col
        let lines: Vec<&str> = app.content.lines().collect();
        let mut visual_row: usize = 0;
        let width = content_w as usize;

        for i in 0..app.cursor_row.min(lines.len()) {
            let len = lines[i].chars().count();
            // number of wrapped segments = ceil(len / width), but at least 1 even when len == 0
            let segs = if width == 0 {
                1
            } else if len == 0 {
                1
            } else {
                (len - 1) / width + 1
            };
            visual_row += segs;
        }

        let visual_col: usize = if app.cursor_row < lines.len() && width > 0 {
            visual_row += app.cursor_col / width;
            app.cursor_col % width
        } else {
            0
        };

        // Ensure caret stays visible by updating scroll window in Edit mode
        if height > 0 {
            if visual_row < start {
                app.view_scroll = visual_row;
                start = visual_row;
            } else if visual_row >= start + height {
                let new_start = visual_row - height + 1;
                app.view_scroll = new_start;
                start = new_start;
            }
        }

        // Apply vertical scroll offset to cursor row so it stays aligned within the visible window
        let cy_row = visual_row.saturating_sub(start);
        // Add line number width offset to cursor x position
        let cx = inner_x + line_num_width + visual_col as u16;
        let cy = inner_y + (cy_row as u16);
        f.set_cursor_position((cx, cy));
    }
}

pub fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    // Detect platform for key binding display
    let save_key = if cfg!(target_os = "macos") {
        "Cmd+S"
    } else {
        "Ctrl+S"
    };

    let status_text = match app.mode {
        AppMode::Preview => {
            let focus_str = match app.focus {
                Focus::Tree => "Tree",
                Focus::Content => "Content",
            };
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
                .title("Help"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(status_paragraph, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    horizontal[1]
}

pub fn draw_date_prompt(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, f.area());
    let mut lines = vec![Line::from("Create entry for date (YYYYMMDD):")];
    let mut input_line = String::from("> ");
    input_line.push_str(&app.date_input);
    lines.push(Line::from(input_line));
    if let Some(err) = &app.date_error {
        lines.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    let block = Block::default()
        .title("New Entry")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let p = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    let clear = Clear;
    f.render_widget(clear, area);
    f.render_widget(p, area);
}

pub fn draw_save_prompt(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 25, f.area());
    let options = ["Yes", "No", "Cancel"];
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("Save changes? "));
    for (i, opt) in options.iter().enumerate() {
        if i == app.save_choice {
            spans.push(Span::styled(
                format!("[{}] ", opt),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::raw(format!("{} ", opt)));
        }
    }
    let block = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let p = Paragraph::new(Line::from(spans)).block(block);
    let clear = Clear;
    f.render_widget(clear, area);
    f.render_widget(p, area);
}
