use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, AppMode, Focus};
use crate::markdown::render_markdown_simple;

pub fn draw_content_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let title = get_panel_title(app);

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
    let content_string = app.content.clone(); // Clone to avoid borrow issues
    let text: Vec<Line> = if app.files.is_empty() && app.current_path.is_none() {
        vec![
            Line::from("No entries."),
            Line::from("Press n to create today's entry."),
        ]
    } else {
        if matches!(app.mode, AppMode::Preview) {
            render_markdown_simple(&content_string, content_w as usize)
        } else {
            render_edit_mode(&content_string, content_w, line_num_width)
        }
    };

    // Apply scrolling and handle cursor positioning while we have mutable access
    handle_cursor_scrolling(app, inner_h, content_w);
    let visible_text = apply_scrolling(app, &text, inner_h);

    // Visual hint for focus: highlight border when Content panel is active (Preview+Content or Edit mode)
    let content_focused = matches!(app.mode, AppMode::Edit)
        || (matches!(app.mode, AppMode::Preview) && app.focus == Focus::Content);
    let right_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if content_focused { Style::default().fg(Color::Yellow) } else { Style::default() });

    let paragraph = Paragraph::new(visible_text).block(right_block);
    f.render_widget(paragraph, area);

    // Draw scrollbar and cursor (now with immutable references)
    draw_scrollbar(f, area, app, &text, inner_x, inner_y, inner_h, line_num_width, content_w);
    draw_cursor_visual(f, app, inner_x, inner_y, inner_h, line_num_width, content_w);
}

fn get_panel_title(app: &App) -> String {
    match (&app.current_path, app.mode) {
        (Some(p), AppMode::Edit) => format!(
            "EDIT — {}{}",
            p.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            if app.dirty { " — ●" } else { "" }
        ),
        (Some(p), AppMode::Preview) => format!(
            "VIEW — {}",
            p.file_name().and_then(|s| s.to_str()).unwrap_or("")
        ),
        (Some(p), _) => p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
        (None, _) => "No entry".to_string(),
    }
}

fn render_edit_mode(content: &str, content_w: u16, line_num_width: u16) -> Vec<Line<'_>> {
    let mut out: Vec<Line> = Vec::new();
    let width = content_w as usize;
    let content_lines: Vec<&str> = content.split('\n').collect();
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
                    Span::styled(format!("{:>width$} ", "", width = line_num_width_usize), line_num_style)
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
            Span::styled(format!("{:>width$} ", "", width = line_num_width_usize), line_num_style)
        };
        if !buf.is_empty() {
            out.push(Line::from(vec![line_prefix, Span::raw(buf)]));
        } else if raw_line.is_empty() || is_first_segment {
            out.push(Line::from(vec![line_prefix, Span::raw("")]));
        }
    }
    out
}


fn apply_scrolling<'a>(app: &mut App, text: &'a [Line], inner_h: u16) -> Vec<Line<'a>> {
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
    let end = if height == 0 { start } else { (start + height).min(text.len()) };
    if start < end { 
        text[start..end].iter().cloned().collect()
    } else { 
        Vec::new() 
    }
}

fn draw_scrollbar(f: &mut Frame, _area: Rect, app: &App, text: &[Line], inner_x: u16, inner_y: u16, inner_h: u16, line_num_width: u16, content_w: u16) {
    if content_w > 0 {
        let total = text.len();
        let view_h = inner_h as usize;
        if total > view_h && view_h > 0 {
            // Position scrollbar at the very right edge (after any line numbers + content)
            let bar_x = inner_x + line_num_width + content_w;
            let bar_area = Rect { x: bar_x, y: inner_y, width: 1, height: inner_h };

            // Track uses interior space between caps (top/bottom). If height is too small, fall back.
            let has_caps = view_h >= 3;
            let track_h = if has_caps { view_h.saturating_sub(2) } else { view_h };

            // Compute thumb size and position within the track
            let thumb_h = ((track_h * track_h) / total).max(1).min(track_h.max(1));
            let max_top = track_h.saturating_sub(thumb_h);
            let denom = total.saturating_sub(view_h).max(1);
            let thumb_top = if total > view_h { (app.view_scroll * max_top) / denom } else { 0 };

            // Build scrollbar glyphs line-by-line
            let mut lines: Vec<Line> = Vec::with_capacity(view_h);
            for i in 0..view_h {
                if has_caps && i == 0 {
                    lines.push(Line::from(Span::styled("▲", Style::default().fg(Color::DarkGray))));
                    continue;
                }
                if has_caps && i == view_h - 1 {
                    lines.push(Line::from(Span::styled("▼", Style::default().fg(Color::DarkGray))));
                    continue;
                }

                let track_i = if has_caps { i - 1 } else { i };
                if track_i >= thumb_top && track_i < thumb_top + thumb_h {
                    lines.push(Line::from(Span::styled("█", Style::default().fg(Color::Gray))));
                } else {
                    lines.push(Line::from(Span::styled("│", Style::default().fg(Color::DarkGray))));
                }
            }
            let sb = Paragraph::new(lines);
            f.render_widget(sb, bar_area);
        }
    }
}

fn handle_cursor_scrolling(app: &mut App, inner_h: u16, content_w: u16) {
    // Handle cursor-based scrolling in edit mode
    if matches!(app.mode, AppMode::Edit) {
        let lines: Vec<&str> = app.content.lines().collect();
        let mut visual_row: usize = 0;
        let width = content_w as usize;

        for i in 0..app.cursor_row.min(lines.len()) {
            let len = lines[i].chars().count();
            let segs = if width == 0 {
                1
            } else if len == 0 {
                1
            } else {
                (len - 1) / width + 1
            };
            visual_row += segs;
        }

        if app.cursor_row < lines.len() {
            let current_line = lines[app.cursor_row];
            let cursor_pos = app.cursor_col.min(current_line.chars().count());
            let segs_before_cursor = if width == 0 { 0 } else { cursor_pos / width };
            visual_row += segs_before_cursor;
        }

        let height = inner_h as usize;
        if height > 0 {
            if visual_row < app.view_scroll {
                app.view_scroll = visual_row;
            } else if visual_row >= app.view_scroll + height {
                let new_start = visual_row.saturating_sub(height).saturating_add(1);
                app.view_scroll = new_start;
            }
        }
    }
}

fn draw_cursor_visual(f: &mut Frame, app: &App, inner_x: u16, inner_y: u16, inner_h: u16, line_num_width: u16, content_w: u16) {
    // Draw cursor visual indicator
    if matches!(app.mode, AppMode::Edit) {
        let lines: Vec<&str> = app.content.lines().collect();
        let mut visual_row: usize = 0;
        let width = content_w as usize;

        for i in 0..app.cursor_row.min(lines.len()) {
            let len = lines[i].chars().count();
            let segs = if width == 0 {
                1
            } else if len == 0 {
                1
            } else {
                (len - 1) / width + 1
            };
            visual_row += segs;
        }

        if app.cursor_row < lines.len() {
            let current_line = lines[app.cursor_row];
            let cursor_pos = app.cursor_col.min(current_line.chars().count());
            let segs_before_cursor = if width == 0 { 0 } else { cursor_pos / width };
            visual_row += segs_before_cursor;

            if visual_row >= app.view_scroll && visual_row < app.view_scroll + inner_h as usize {
                let display_row = visual_row - app.view_scroll;
                let col_in_segment = if width == 0 { 0 } else { cursor_pos % width };
                let cursor_x = inner_x + line_num_width + col_in_segment as u16;
                let cursor_y = inner_y + display_row as u16;

                if cursor_x < inner_x + line_num_width + content_w && cursor_y < inner_y + inner_h {
                    f.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
    }
}


