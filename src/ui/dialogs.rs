use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;

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
    let p = Paragraph::new(lines).block(block).alignment(Alignment::Left);
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
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
