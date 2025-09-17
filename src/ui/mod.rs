use std::io::{self, stdout};

use color_eyre::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    prelude::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
};

pub fn run() -> Result<()> {
    // Normal Mode vs Raw Mode
    // ** Normal Mode **
    // - Terminal processes special keys: `ctrl+c` kills program, `enter` submits line
    // - Text is buffered till you press Enter.
    // - Terminal handles cursor movement, backspace
    // ** Raw Mode **
    // - Program gets every single keypress immediately
    // - No special key processing (`ctrl+c` won't kill our program)
    // - No line buffering
    // - Our program contls the entire screen
    // - Terminal doesn't echo what we type

    // 1. Switch terminal to raw mode
    enable_raw_mode()?;

    // 2. Switch to alternate screen (like vim/less do)
    // Save the current terminal content and start the app with a clean screen
    stdout().execute(EnterAlternateScreen)?;
    //  ↑        ↑           ↑            ↑
    //  │        │           │            └─ Propagate any errors
    //  │        │           └─ Command to switch to alternate screen
    //  │        └─ Execute a terminal command
    //  └─ Get the standard output stream

    // 3. Create the terminal interface
    // stdout() -> CrosstermBackend -> Terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 4. Run the app
    let result = run_app(&mut terminal);

    // 5. Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    loop {
        // Draw frame
        terminal.draw(|frame| {
            let area = frame.area();
            let paragraph = Paragraph::new("Hello user! Press 'q' to quit")
                .block(Block::default().title("DevLog").borders(Borders::ALL));
            frame.render_widget(paragraph, area);
        })?;

        // Wait for keypress
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}
