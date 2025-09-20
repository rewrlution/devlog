use std::io;
use std::time::Duration;
use std::env;

mod ai;
mod app;
mod ui;
mod events;
mod markdown;
mod utils;
mod ai_mode;

use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use events::handle_key;
use ui::ui;
use ai_mode::run_ai_mode;

fn main() -> io::Result<()> {
    // Subcommand dispatch before entering TUI
    let mut args = env::args();
    let _exe = args.next();
    if let Some(cmd) = args.next() {
        if cmd == "ai" {
            // Run AI REPL and exit
            return run_ai_mode();
        }
    }

    // Default: run TUI app
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new()?;
    let tick_rate = Duration::from_millis(200);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(app.last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key)? {
                    break Ok(());
                }
            }
        }
        if app.last_tick.elapsed() >= tick_rate {
            app.last_tick = std::time::Instant::now();
        }
    }
}


