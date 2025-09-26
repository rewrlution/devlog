use std::io;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    init,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
};

pub struct App {
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            // Draw the UI
            terminal.draw(|frame| {
                let area = frame.area();

                let paragraph = Paragraph::new("Hello World!\n\nPress 'q' to quit")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("devlog 1.0"));

                frame.render_widget(paragraph, area);
            })?;

            // Handle events
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    self.should_quit = true;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}

pub fn launch_tui() -> Result<()> {
    // `raw mode` disables the terminal's default line-buffered input processing
    // `EnterAlternateScreen` starts a completely clean screen
    // `LeaveAlternateScreen` goes back to the original state
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // `terminal` is ratatui's abstraction for drawing to the screen.
    // It manages the screen buffer, handles drawing operations, and coordinates with the backend
    // `app` is the logic and the state of our application.
    // It handles events, and maintains app states.
    let mut terminal = init();
    let mut app = App::new();

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    result
}
