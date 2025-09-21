use crate::tui::data::AppState;
use crate::tui::events::EventHandler;
use crate::tui::tree_builder::{flatten_tree, TreeBuilder};
use crate::tui::ui::UIRenderer;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use std::io;

pub struct App {
    app_state: AppState,
    tree_state: ListState,
    event_handler: EventHandler,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app_state = AppState::new();
        let tree_builder = TreeBuilder::new()?;

        // Load the tree
        app_state.tree_nodes = tree_builder.build_tree()?;
        app_state.flat_items = flatten_tree(&app_state.tree_nodes);

        let mut tree_state = ListState::default();
        tree_state.select(Some(0));

        let event_handler = EventHandler::new()?;

        Ok(Self {
            app_state,
            tree_state,
            event_handler,
        })
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Clear terminal if a redraw is needed (e.g., after editor)
            if self.app_state.needs_redraw {
                terminal.clear()?;
                self.app_state.needs_redraw = false;
            }

            terminal.draw(|f| UIRenderer::render(&self.app_state, &mut self.tree_state, f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.event_handler.handle_key_event(
                        key.code,
                        &mut self.app_state,
                        &mut self.tree_state,
                    )?;
                }
            }

            if self.app_state.should_quit {
                break;
            }
        }
        Ok(())
    }
}

// Helper function to launch TUI
pub fn launch_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = app.run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
