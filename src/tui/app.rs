use std::io;

use color_eyre::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{init, widgets::ListState, DefaultTerminal};

use crate::{
    storage::Storage,
    tui::{
        handlers::keyboard::KeyboardHandler,
        models::state::AppState,
        tree::{builder::TreeBuilder, flattener::TreeFlattener},
        ui::UIRenderer,
    },
};

pub struct App {
    app_state: AppState,
    tree_state: ListState,
    keyboard_handler: KeyboardHandler,
}

impl App {
    pub fn new(storage: &Storage) -> Result<Self> {
        let tree_builder = TreeBuilder::new(storage.clone());
        let tree_nodes = tree_builder.build_tree()?;
        let flat_items = TreeFlattener::flatten(&tree_nodes);

        let mut app_state = AppState::new();
        app_state.tree_nodes = tree_nodes;
        app_state.flat_items = flat_items;

        Ok(Self {
            app_state,
            tree_state: ListState::default(),
            keyboard_handler: KeyboardHandler::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            // Draw the UI
            terminal.draw(|f| UIRenderer::render(&self.app_state, &mut self.tree_state, f))?;

            // Handle events
            if let Event::Key(key) = event::read()? {
                self.keyboard_handler.handle_key_event(
                    key.code,
                    &mut self.app_state,
                    &mut self.tree_state,
                )?;
            }

            if self.app_state.should_quit {
                break;
            }
        }

        Ok(())
    }
}

pub fn launch_tui(storage: &Storage) -> Result<()> {
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
    let mut app = App::new(storage)?;

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    result
}
