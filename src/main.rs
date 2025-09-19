use std::cmp::min;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::env;
use std::collections::BTreeMap;
use pulldown_cmark::{Event as MdEvent, Parser, Tag, TagEnd};

mod ai;
use ai::{AiConfig, ask_question, create_client, load_devlog_context, read_ai_config};

use chrono::{Datelike, NaiveDate};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::{Frame, Terminal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppMode {
    Preview,
    Edit,
    DatePrompt,
    SavePrompt,
}

fn run_ai_mode() -> io::Result<()> {
    // Read config
    let devlog_path = devlog_path();
    let cfg = read_ai_config(&devlog_path)?;
    let api_key = env::var("OPENAI_API_KEY")
        .ok()
        .or(cfg.openai_api_key)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "OpenAI API key not set. Set OPENAI_API_KEY env or write openai_api_key in .devlog/config.toml",
            )
        })?;
    let model = cfg.model.unwrap_or_else(|| "gpt-4o-mini".to_string());

    // Initialize client and load context
    let client = create_client(&api_key);
    let context = load_devlog_context(&devlog_path, 200_000).unwrap_or_default();

    println!("devlog ai — ask about files in .devlog (type 'exit' to quit)\n");
    let system_prefix = "You are a helpful assistant that answers questions about the user's devlog notes. Base your answers strictly on the provided files. If unsure, say you don't know.";
    let full_context = format!("{}\n\nHere are the devlog files:\n{}", system_prefix, context);

    // Simple REPL loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // Create a single runtime for the entire session
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Tokio runtime error: {}", e)))?;

    loop {
        print!(">> ");
        let _ = stdout.flush();
        let mut q = String::new();
        if stdin.read_line(&mut q)? == 0 {
            break;
        }
        let q = q.trim();
        if q.is_empty() {
            continue;
        }
        if q.eq_ignore_ascii_case("exit") || q.eq_ignore_ascii_case("quit") {
            break;
        }

        match rt.block_on(ask_question(&client, &model, &full_context, q)) {
            Ok(response) => println!("\n{}\n", response.trim()),
            Err(e) => println!("\nError: {}\n", e),
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Tree,
    Content,
}

#[derive(Clone, Debug)]
enum NodeKind {
    Year,
    Month,
    Day { filename: String },
}

#[derive(Clone, Debug)]
struct TreeNode {
    label: String,
    kind: NodeKind,
    children: Vec<TreeNode>,
    expanded: bool,
}

struct App {
    // Source files and derived tree
    files: Vec<String>,
    tree_root: Vec<TreeNode>,
    // Flattened visible nodes for rendering and selection
    flat_nodes: Vec<(usize, Vec<usize>)>, // (indent, path indices)
    selected_index: Option<usize>,
    // Currently open file (full path) and its content
    current_path: Option<PathBuf>,
    content: String,
    // Editor state
    cursor_row: usize,
    cursor_col: usize,
    // View state
    focus: Focus,
    view_scroll: usize,
    dirty: bool,
    mode: AppMode,
    // Date prompt state
    date_input: String,
    date_error: Option<String>,
    // Save prompt state: 0=Yes,1=No,2=Cancel
    save_choice: usize,
    // Timing
    last_tick: Instant,
}

impl App {
    fn new() -> io::Result<Self> {
        let mut app = Self {
            files: list_existing_devlog_files()?,
            tree_root: Vec::new(),
            flat_nodes: Vec::new(),
            selected_index: None,
            current_path: None,
            content: String::new(),
            cursor_row: 0,
            cursor_col: 0,
            focus: Focus::Tree,
            view_scroll: 0,
            dirty: false,
            mode: AppMode::Preview,
            date_input: today_str(),
            date_error: None,
            save_choice: 0,
            last_tick: Instant::now(),
        };
        // Build tree and select most recent file if any
        app.rebuild_tree();
        if !app.files.is_empty() {
            if let Some(name) = app.files.last().cloned() {
                app.select_day_by_filename(&name);
                app.open_file_by_name(&name).ok();
            }
        }
        Ok(app)
    }

    fn selected_filename(&self) -> Option<&str> {
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel) {
                if let Some(node) = self.node_by_path(path) {
                    if let NodeKind::Day { filename } = &node.kind {
                        return Some(filename.as_str());
                    }
                }
            }
        }
        None
    }

    fn open_file_by_name(&mut self, name: &str) -> io::Result<()> {
        let path = devlog_path().join(name);
        let mut f = File::open(&path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        self.current_path = Some(path);
        self.content = content;
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.view_scroll = 0; // reset scroll when changing file
        self.dirty = false;
        self.mode = AppMode::Preview;
        Ok(())
    }

    fn open_or_create_for_date(&mut self, yyyymmdd: &str) -> io::Result<()> {
        let name = format!("{}.md", yyyymmdd);
        let path = devlog_path().join(&name);
        fs::create_dir_all(devlog_path())?;
        if !path.exists() {
            File::create(&path)?; // create empty file
            // refresh file list and tree
            self.files = list_existing_devlog_files()?;
            self.rebuild_tree();
        }
        // select the day node in tree
        self.select_day_by_filename(&name);
        // open and switch to edit
        self.open_file_by_name(&name)?;
        self.mode = AppMode::Edit;
        self.move_cursor_to_end();
        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.current_path {
            let mut f = File::create(path)?;
            f.write_all(self.content.as_bytes())?;
            self.dirty = false;
        }
        Ok(())
    }

    // ---- Tree building and navigation helpers ----
    fn rebuild_tree(&mut self) {
        let mut root: Vec<TreeNode> = Vec::new();
        let mut year_map: BTreeMap<i32, BTreeMap<u32, Vec<String>>> = BTreeMap::new();
        
        // Find the latest entry filename
        let latest_file = self.files.iter().max().cloned();
        let (latest_year, latest_month) = if let Some(ref fname) = latest_file {
            if let Ok(date) = NaiveDate::parse_from_str(&fname[..8], "%Y%m%d") {
                (Some(date.year()), Some(date.month()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        
        // Group files by year and month
        for fname in &self.files {
            if let Ok(date) = NaiveDate::parse_from_str(&fname[..8], "%Y%m%d") {
                let year = date.year();
                let month = date.month();
                year_map.entry(year).or_default().entry(month).or_default().push(fname.clone());
            }
        }
        
        // Build tree structure - iterate years in descending order
        for (year, months) in year_map.into_iter().rev() {
            let mut year_node = TreeNode {
                label: format!("{}", year),
                kind: NodeKind::Year,
                children: Vec::new(),
                expanded: Some(year) == latest_year, // Only expand year containing latest entry
            };
            for (month, mut days) in months.into_iter().rev() {
                let mut month_node = TreeNode {
                    label: format!("{:04}-{:02}", year, month),
                    kind: NodeKind::Month,
                    children: Vec::new(),
                    expanded: Some(year) == latest_year && Some(month) == latest_month, // Only expand month containing latest entry
                };
                // Sort days in descending order (newest first)
                days.sort_by(|a, b| b.cmp(a));
                for fname in days {
                    let date = &fname[..8];
                    let label = match NaiveDate::parse_from_str(date, "%Y%m%d") {
                        Ok(d) => d.format("%Y-%m-%d").to_string(),
                        Err(_) => date.to_string(),
                    };
                    month_node.children.push(TreeNode {
                        label,
                        kind: NodeKind::Day {
                            filename: fname,
                        },
                        children: Vec::new(),
                        expanded: false,
                    });
                }
                year_node.children.push(month_node);
            }
            root.push(year_node);
        }
        self.tree_root = root;
        self.recompute_flat_nodes();
        
        // Auto-select the latest entry
        if let Some(latest_file) = latest_file {
            self.select_day_by_filename(&latest_file);
            // Open the latest file
            let _ = self.open_file_by_name(&latest_file);
        } else if self.selected_index.is_none() && !self.flat_nodes.is_empty() {
            self.selected_index = Some(0);
        }
    }

    fn recompute_flat_nodes(&mut self) {
        self.flat_nodes.clear();
        let len = self.tree_root.len();
        for i in 0..len {
            let mut path = vec![i];
            self.flatten_from(&mut path, 0);
        }
        if let Some(sel) = self.selected_index {
            if self.flat_nodes.is_empty() {
                self.selected_index = None;
            } else if sel >= self.flat_nodes.len() {
                self.selected_index = Some(self.flat_nodes.len() - 1);
            }
        }
    }

    fn flatten_from(&mut self, path: &mut Vec<usize>, indent: usize) {
        self.flat_nodes.push((indent, path.clone()));
        if let Some(node) = self.node_by_path(path) {
            if node.expanded {
                for child_idx in 0..node.children.len() {
                    path.push(child_idx);
                    self.flatten_from(path, indent + 1);
                    path.pop();
                }
            }
        }
    }

    fn is_last_child(&self, path: &[usize]) -> bool {
        if path.is_empty() {
            return false;
        }
        let parent_path = &path[..path.len() - 1];
        let child_index = path[path.len() - 1];
        
        if let Some(parent) = self.node_by_path_slice(parent_path) {
            child_index == parent.children.len() - 1
        } else if parent_path.is_empty() {
            // Root level
            child_index == self.tree_root.len() - 1
        } else {
            false
        }
    }

    fn node_by_path(&self, path: &Vec<usize>) -> Option<&TreeNode> {
        self.node_by_path_slice(path.as_slice())
    }

    fn node_by_path_slice(&self, path: &[usize]) -> Option<&TreeNode> {
        let mut cur: Option<&TreeNode> = None;
        for (depth, &idx) in path.iter().enumerate() {
            if depth == 0 {
                cur = self.tree_root.get(idx);
            } else {
                cur = cur.and_then(|n| n.children.get(idx));
            }
        }
        cur
    }

    fn node_by_path_mut(&mut self, path: &Vec<usize>) -> Option<&mut TreeNode> {
        self.node_by_path_mut_slice(path.as_slice())
    }

    fn node_by_path_mut_slice(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let (first, rest) = (path[0], &path[1..]);
        let node = self.tree_root.get_mut(first)?;
        if rest.is_empty() {
            return Some(node);
        }
        let mut cur = node;
        for &idx in rest {
            cur = cur.children.get_mut(idx)?;
        }
        Some(cur)
    }

    fn toggle_expand_at_selected(&mut self, expand: bool) {
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel).cloned() {
                if let Some(node) = self.node_by_path_mut(&path) {
                    if !matches!(node.kind, NodeKind::Day { .. }) {
                        node.expanded = expand;
                        self.recompute_flat_nodes();
                    }
                }
            }
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.flat_nodes.is_empty() {
            self.selected_index = None;
            return;
        }
        let cur = self.selected_index.unwrap_or(0) as isize;
        let len = self.flat_nodes.len() as isize;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next >= len {
            next = len - 1;
        }
        self.selected_index = Some(next as usize);
        // Only auto-open files if we're navigating to a day node
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel) {
                if let Some(node) = self.node_by_path(path) {
                    if let NodeKind::Day { filename } = &node.kind {
                        let filename_clone = filename.clone();
                        let _ = self.open_file_by_name(&filename_clone);
                    }
                }
            }
        }
    }

    fn select_day_by_filename(&mut self, filename: &str) {
        for (i, (_indent, path)) in self.flat_nodes.iter().enumerate() {
            if let Some(node) = self.node_by_path(path) {
                if let NodeKind::Day { filename: f } = &node.kind {
                    if f == filename {
                        self.selected_index = Some(i);
                        return;
                    }
                }
            }
        }
    }

    fn validate_date(input: &str) -> Result<(), &'static str> {
        if input.len() != 8 || !input.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid date. Use YYYYMMDD.");
        }
        if NaiveDate::parse_from_str(input, "%Y%m%d").is_err() {
            return Err("Invalid calendar date.");
        }
        Ok(())
    }

    fn move_cursor_to_end(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            self.cursor_row = 0;
            self.cursor_col = 0;
        } else {
            self.cursor_row = lines.len() - 1;
            // Use character count, not byte length
            self.cursor_col = lines.last().unwrap().chars().count();
        }
    }

    fn insert_char(&mut self, ch: char) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());
        
        // Insert character at the correct character position
        let mut new_chars = line_chars;
        new_chars.insert(col, ch);
        *line = new_chars.into_iter().collect();
        
        // Advance cursor by 1 character (not bytes)
        self.cursor_col = col + 1;
        self.content = lines.join("\n");
        self.dirty = true;
    }

    fn backspace(&mut self) {
        if self.cursor_row == 0 && self.cursor_col == 0 {
            return;
        }
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col > 0 {
            let line = &mut lines[row];
            // Convert to character-based indexing
            let mut line_chars: Vec<char> = line.chars().collect();
            if col <= line_chars.len() {
                let char_idx = col - 1;
                line_chars.remove(char_idx);
                *line = line_chars.into_iter().collect();
                self.cursor_col = char_idx;
            }
        } else if row > 0 {
            // Moving to previous line - use character count for cursor position
            let prev_line_chars = lines[row - 1].chars().count();
            let current = lines.remove(row);
            self.cursor_row -= 1;
            self.cursor_col = prev_line_chars;
            lines[self.cursor_row].push_str(&current);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    fn delete(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let mut line_chars: Vec<char> = line.chars().collect();
        let line_char_len = line_chars.len();
        
        if self.cursor_col < line_char_len {
            // Delete character at cursor position
            line_chars.remove(self.cursor_col);
            *line = line_chars.into_iter().collect();
        } else if row + 1 < lines.len() {
            // Delete newline - merge with next line
            let next = lines.remove(row + 1);
            lines[row].push_str(&next);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    fn insert_newline(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        
        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());
        
        // Split line at character position
        let (left_chars, right_chars) = line_chars.split_at(col);
        *line = left_chars.iter().collect();
        let rest: String = right_chars.iter().collect();
        
        self.cursor_row = row + 1;
        self.cursor_col = 0;
        lines.insert(self.cursor_row, rest);
        self.content = lines.join("\n");
        self.dirty = true;
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            self.cursor_col = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
        }
    }

    fn move_right(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            return;
        }
        // Use character count, not byte length
        let line_char_len = lines[self.cursor_row.min(lines.len() - 1)].chars().count();
        if self.cursor_col < line_char_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            let line_char_len = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }

    fn move_down(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            // Use character count, not byte length
            let line_char_len = lines[self.cursor_row].chars().count();
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }
}

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
            app.last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
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

fn draw_left(f: &mut Frame, area: Rect, app: &mut App) {
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

fn draw_right(f: &mut Frame, area: Rect, app: &mut App) {
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
        (Some(p), _) => p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
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
    let end = if height == 0 { start } else { (start + height).min(text.len()) };
    let visible: Vec<Line> = if start < end { text[start..end].to_vec() } else { Vec::new() };

    // Visual hint for focus: highlight border when Content panel is active (Preview+Content or Edit mode)
    let content_focused = matches!(app.mode, AppMode::Edit)
        || (matches!(app.mode, AppMode::Preview) && app.focus == Focus::Content);
    let right_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if content_focused { Style::default().fg(Color::Yellow) } else { Style::default() });

    let paragraph = Paragraph::new(visible).block(right_block);
    f.render_widget(paragraph, area);

    // Draw vertical scrollbar in reserved 1-column strip when there is overflow
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
            let thumb_top = if total > view_h { (start * max_top) / denom } else { 0 };

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

// Very simple Markdown renderer suitable for Preview mode. It renders block-level
// structures (headings, lists, paragraphs, code blocks) with basic styling, and
// pre-wraps at the given width so scrolling and the scrollbar remain correct.
fn render_markdown_simple(src: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<(String, Style)> = Vec::new();
    let mut cur = String::new();
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MdBlock { Paragraph, Heading(u32), Code, ListItem }
    let mut block = MdBlock::Paragraph;

    // Local style mapper to avoid name/type conflicts
    fn style_for(block: &MdBlock) -> Style {
        match block {
            MdBlock::Heading(_) => Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            MdBlock::Code => Style::default().fg(Color::Gray),
            MdBlock::ListItem => Style::default(),
            MdBlock::Paragraph => Style::default(),
        }
    }

    let parser = Parser::new(src);
    for ev in parser {
        match ev {
            MdEvent::Start(Tag::Heading { level, .. }) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Heading(level as u32);
            }
            MdEvent::End(TagEnd::Heading(_)) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
                lines.push((String::new(), Style::default()));
            }
            MdEvent::Start(Tag::CodeBlock(_)) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Code;
            }
            MdEvent::End(TagEnd::CodeBlock) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
                lines.push((String::new(), Style::default()));
            }
            MdEvent::Start(Tag::List(_)) => {}
            MdEvent::End(TagEnd::List(_)) => {}
            MdEvent::Start(Tag::Item) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::ListItem;
                cur.push_str("• ");
            }
            MdEvent::End(TagEnd::Item) => {
                if !cur.is_empty() { lines.push((cur.clone(), style_for(&block))); cur.clear(); }
                block = MdBlock::Paragraph;
            }
            MdEvent::Text(t) | MdEvent::Code(t) => {
                cur.push_str(&t);
            }
            MdEvent::SoftBreak | MdEvent::HardBreak => {
                lines.push((cur.clone(), style_for(&block)));
                cur.clear();
            }
            _ => {}
        }
    }
    if !cur.is_empty() { lines.push((cur, style_for(&block))); }

    // Wrap to width, applying the same style to each wrapped segment
    let mut out: Vec<Line> = Vec::new();
    for (mut s, st) in lines {
        if width == 0 {
            out.push(Line::from(Span::styled(s, st)));
            continue;
        }
        while !s.is_empty() {
            let take = s.chars().take(width).collect::<String>();
            let rem_len = s.chars().count();
            out.push(Line::from(Span::styled(take.clone(), st)));
            if rem_len <= width { break; }
            s = s.chars().skip(width).collect();
        }
    }
    out
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn draw_date_prompt(f: &mut Frame, app: &App) {
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

fn draw_save_prompt(f: &mut Frame, app: &App) {
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

fn handle_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match app.mode {
        AppMode::Preview => handle_key_preview(app, key),
        AppMode::Edit => handle_key_edit(app, key),
        AppMode::DatePrompt => handle_key_date_prompt(app, key),
        AppMode::SavePrompt => handle_key_save_prompt(app, key),
    }
}

fn handle_key_preview(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('n') => {
            app.date_input = today_str();
            app.date_error = None;
            app.mode = AppMode::DatePrompt;
        }
        KeyCode::Char('e') => {
            if app.current_path.is_some() {
                app.mode = AppMode::Edit;
                app.move_cursor_to_end();
            }
        }
        KeyCode::Tab => {
            // Toggle focus between tree and content in preview mode
            app.focus = if app.focus == Focus::Tree { Focus::Content } else { Focus::Tree };
        }
        KeyCode::Up => {
            if app.focus == Focus::Tree {
                app.move_selection(-1);
            } else {
                app.view_scroll = app.view_scroll.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if app.focus == Focus::Tree {
                app.move_selection(1);
            } else {
                app.view_scroll = app.view_scroll.saturating_add(1);
            }
        }
        KeyCode::Left => {
            app.toggle_expand_at_selected(false);
        }
        KeyCode::Right => {
            app.toggle_expand_at_selected(true);
        }
        KeyCode::Enter => {
            // If a file is selected, open it for viewing
            if let Some(name) = app.selected_filename().map(|s| s.to_string()) {
                let _ = app.open_file_by_name(&name);
                app.focus = Focus::Content;
            }
        }
        KeyCode::Esc => return Ok(true), // quit app from preview with Esc
        _ => {}
    }
    Ok(false)
}

fn handle_key_edit(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key {
        // Cross-platform save: Accept both Ctrl+S and Cmd+S
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers,
            ..
        } if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::SUPER) => {
            app.save()?;
        }
        KeyEvent { code: KeyCode::Esc, .. } => {
            if app.dirty {
                app.mode = AppMode::SavePrompt;
                app.save_choice = 0;
            } else {
                app.mode = AppMode::Preview;
            }
        }
        KeyEvent { code: KeyCode::Left, .. } => app.move_left(),
        KeyEvent { code: KeyCode::Right, .. } => app.move_right(),
        KeyEvent { code: KeyCode::Up, .. } => app.move_up(),
        KeyEvent { code: KeyCode::Down, .. } => app.move_down(),
        KeyEvent { code: KeyCode::Backspace, .. } => app.backspace(),
        KeyEvent { code: KeyCode::Delete, .. } => app.delete(),
        KeyEvent { code: KeyCode::Enter, .. } => app.insert_newline(),
        KeyEvent { code: KeyCode::Tab, .. } => {
            // insert 2 spaces as tab
            app.insert_char(' ');
            app.insert_char(' ');
        }
        KeyEvent { code: KeyCode::Char(c), .. } => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                app.insert_char(c);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_key_date_prompt(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Preview;
        }
        KeyCode::Enter => {
            match App::validate_date(&app.date_input) {
                Ok(()) => {
                    app.date_error = None;
                    let date = app.date_input.clone();
                    app.open_or_create_for_date(&date)?;
                }
                Err(msg) => {
                    app.date_error = Some(msg.to_string());
                }
            }
        }
        KeyCode::Backspace => {
            app.date_input.pop();
            app.date_error = None;
        }
        KeyCode::Char(c) => {
            if c.is_ascii_digit() && app.date_input.len() < 8 {
                app.date_input.push(c);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_key_save_prompt(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Left => {
            if app.save_choice > 0 {
                app.save_choice -= 1;
            }
        }
        KeyCode::Right => {
            if app.save_choice < 2 {
                app.save_choice += 1;
            }
        }
        KeyCode::Enter => match app.save_choice {
            0 => {
                app.save()?;
                app.mode = AppMode::Preview;
            }
            1 => {
                // discard: reload from disk
                if let Some(path) = &app.current_path {
                    let mut s = String::new();
                    File::open(path)?.read_to_string(&mut s)?;
                    app.content = s;
                    app.dirty = false;
                }
                app.mode = AppMode::Preview;
            }
            _ => {
                // Cancel
                app.mode = AppMode::Edit;
            }
        },
        KeyCode::Esc => {
            app.mode = AppMode::Edit;
        }
        _ => {}
    }
    Ok(false)
}

fn devlog_path() -> PathBuf {
    // 1) Allow override through environment variable
    if let Ok(dir) = env::var("DEVLOG_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    // 2) Search upwards from the current working directory for a `.devlog/` folder
    if let Ok(mut cur) = env::current_dir() {
        loop {
            let candidate = cur.join(".devlog");
            if candidate.is_dir() {
                return candidate;
            }
            if !cur.pop() {
                break; // reached filesystem root
            }
        }
    }

    // 3) Fallback to relative `.devlog/` in the current directory
    Path::new(".devlog").to_path_buf()
}

fn list_existing_devlog_files() -> io::Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let path = devlog_path();
    if !path.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        // Only consider regular files
        if entry.file_type()?.is_file() {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy().to_string();
            if is_valid_entry_filename(&file_name) {
                out.push(file_name);
            }
        }
    }
    // sort by filename (YYYYMMDD.md) descending (newest first)
    out.sort_by(|a, b| b.cmp(a));
    Ok(out)
}

fn is_valid_entry_filename(name: &str) -> bool {
    if name.len() != 11 || !name.ends_with(".md") {
        return false;
    }
    let date = &name[..8];
    date.chars().all(|c| c.is_ascii_digit())
}

fn today_str() -> String {
    let now = chrono::Local::now().naive_local().date();
    now.format("%Y%m%d").to_string()
}
