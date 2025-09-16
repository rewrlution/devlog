# Testing Strategy

## Overview

This document outlines a comprehensive testing strategy for the Engineering Journal app, designed to be beginner-friendly while ensuring code quality and reliability.

## Testing Philosophy

### Start Simple, Build Up

1. **Unit tests first**: Test individual functions and modules
2. **Integration tests**: Test how modules work together
3. **End-to-end tests**: Test complete user workflows
4. **Manual testing**: Regular testing of the actual TUI

### Rust Testing Basics

#### Built-in Test Framework

```rust
// src/storage/entry.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_entry_creation() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let entry = Entry::new(date);

        assert_eq!(entry.date, date);
        assert!(entry.content.is_none());
    }

    #[test]
    fn test_entry_path_generation() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let entry = Entry::new(date);

        let expected_filename = "20250315.md";
        assert!(entry.path.to_string_lossy().ends_with(expected_filename));
    }
}
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_entry_creation

# Run tests in specific module
cargo test storage::tests

# Run tests and show coverage
cargo test --verbose
```

## Unit Testing Strategy

### 1. Data Structure Tests

#### Entry Management

```rust
// src/storage/tests.rs
use super::*;
use tempfile::TempDir;

#[test]
fn test_entry_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    let content = "# Daily Entry\n\nToday I learned Rust!".to_string();

    let mut entry = Entry::new(date);
    entry.content = Some(content.clone());
    entry.path = temp_dir.path().join("20250315.md");

    // Test saving
    entry.save().unwrap();
    assert!(entry.path.exists());

    // Test loading
    let mut loaded_entry = Entry::new(date);
    loaded_entry.path = entry.path.clone();
    loaded_entry.load_content().unwrap();

    assert_eq!(loaded_entry.content.as_ref().unwrap(), &content);
}

#[test]
fn test_entry_date_parsing() {
    let test_cases = vec![
        ("20250315", true),   // Valid
        ("2025315", false),   // Invalid: missing zero
        ("20250229", false),  // Invalid: not leap year
        ("20240229", true),   // Valid: leap year
        ("20251301", false),  // Invalid: month > 12
        ("20250132", false),  // Invalid: day > 31
    ];

    for (date_str, should_be_valid) in test_cases {
        let result = parse_entry_date(date_str);
        assert_eq!(result.is_ok(), should_be_valid, "Failed for date: {}", date_str);
    }
}
```

#### Navigation Tests

```rust
// src/navigation/tests.rs
#[test]
fn test_tree_navigation() {
    let mut tree = create_test_tree();

    // Test moving down
    tree.move_down();
    assert_eq!(tree.selected_index(), Some(1));

    // Test wrapping at bottom
    tree.select_last();
    tree.move_down();
    assert_eq!(tree.selected_index(), Some(0));

    // Test moving up
    tree.move_up();
    assert_eq!(tree.selected_index(), Some(tree.items.len() - 1));
}

#[test]
fn test_tree_expand_collapse() {
    let mut tree = create_test_tree();

    // Select a year node
    tree.select_item_by_path("2025");

    // Test expansion
    assert!(!tree.is_expanded("2025"));
    tree.toggle_current();
    assert!(tree.is_expanded("2025"));

    // Test collapse
    tree.toggle_current();
    assert!(!tree.is_expanded("2025"));
}

fn create_test_tree() -> TreeViewState {
    let entries = vec![
        create_test_entry(2025, 3, 15),
        create_test_entry(2025, 3, 16),
        create_test_entry(2025, 4, 1),
        create_test_entry(2024, 12, 31),
    ];

    TreeViewState::from_entries(entries)
}

fn create_test_entry(year: i32, month: u32, day: u32) -> Entry {
    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    Entry::new(date)
}
```

### 2. Editor Tests

```rust
// src/editor/tests.rs
#[test]
fn test_text_insertion() {
    let mut editor = EditorState::new();

    // Insert characters
    editor.insert_char('H');
    editor.insert_char('i');

    assert_eq!(editor.current_line(), "Hi");
    assert_eq!(editor.cursor_col, 2);
}

#[test]
fn test_line_operations() {
    let mut editor = EditorState::with_content(vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
    ]);

    // Insert new line
    editor.cursor_line = 0;
    editor.cursor_col = 6;
    editor.insert_newline();

    assert_eq!(editor.lines.len(), 3);
    assert_eq!(editor.lines[0], "Line 1");
    assert_eq!(editor.lines[1], "");
    assert_eq!(editor.cursor_line, 1);
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_cursor_movement() {
    let mut editor = EditorState::with_content(vec![
        "First line".to_string(),
        "Second line".to_string(),
    ]);

    // Move right
    editor.move_cursor_right();
    assert_eq!(editor.cursor_col, 1);

    // Move to end of line
    editor.cursor_col = editor.current_line().len();
    editor.move_cursor_right(); // Should not move past end
    assert_eq!(editor.cursor_col, editor.current_line().len());

    // Move down
    editor.move_cursor_down();
    assert_eq!(editor.cursor_line, 1);
}
```

## Integration Testing

### 1. File System Integration

```rust
// tests/integration_tests.rs
use engineering_journal::{App, storage::Storage};
use tempfile::TempDir;

#[test]
fn test_full_entry_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let mut app = App::new_with_storage_path(temp_dir.path()).unwrap();

    // Create entry
    let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    let content = "Test entry content".to_string();
    app.create_entry(date, content.clone()).unwrap();

    // Verify entry exists
    assert!(app.storage.entry_exists(&date));

    // Load entry
    let loaded_entry = app.storage.load_entry(&date).unwrap();
    assert_eq!(loaded_entry.content.as_ref().unwrap(), &content);

    // Delete entry
    app.storage.delete_entry(&date).unwrap();
    assert!(!app.storage.entry_exists(&date));
}

#[test]
fn test_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();

    let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    let entry = Entry::new(date);

    storage.save_entry(&entry).unwrap();

    // Verify directory structure
    let year_dir = temp_dir.path().join("entries").join("2025");
    let month_dir = year_dir.join("03");
    let entry_file = month_dir.join("20250315.md");

    assert!(year_dir.exists());
    assert!(month_dir.exists());
    assert!(entry_file.exists());
}
```

### 2. App State Integration

```rust
#[test]
fn test_app_mode_transitions() {
    let mut app = create_test_app();

    // Start in navigation mode
    assert!(matches!(app.mode, AppMode::Navigation));

    // Transition to edit mode
    let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    app.enter_edit_mode(date).unwrap();
    assert!(matches!(app.mode, AppMode::Edit));

    // Return to navigation mode
    app.exit_edit_mode();
    assert!(matches!(app.mode, AppMode::Navigation));
}

#[test]
fn test_entry_creation_workflow() {
    let mut app = create_test_app();

    // Simulate 'n' key press (create today's entry)
    app.handle_create_today_entry().unwrap();

    // Should be in edit mode
    assert!(matches!(app.mode, AppMode::Edit));

    // Should have current date entry
    let today = chrono::Local::now().date_naive();
    assert!(app.storage.entry_exists(&today));
}
```

## UI Testing

### 1. Mocking ratatui Components

```rust
// src/ui/tests.rs
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

#[test]
fn test_tree_view_rendering() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let app = create_test_app_with_entries();

    terminal.draw(|f| {
        ui::render_tree_view(&app, f, f.size());
    }).unwrap();

    let buffer = terminal.backend().buffer();

    // Check that year is displayed
    assert!(buffer_contains_text(buffer, "2025"));

    // Check that tree indicators are present
    assert!(buffer_contains_text(buffer, "▼"));
    assert!(buffer_contains_text(buffer, "▶"));
}

#[test]
fn test_status_bar_content() {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let app = create_test_app();

    terminal.draw(|f| {
        ui::render_status_bar(&app, f, f.size());
    }).unwrap();

    let buffer = terminal.backend().buffer();

    // Check navigation mode status
    assert!(buffer_contains_text(buffer, "[hjkl"));
    assert!(buffer_contains_text(buffer, "Navigate"));
    assert!(buffer_contains_text(buffer, "[q] Quit"));
}

fn buffer_contains_text(buffer: &Buffer, text: &str) -> bool {
    buffer.content().iter().any(|cell| cell.symbol().contains(text))
}
```

### 2. Event Handling Tests

```rust
#[test]
fn test_navigation_key_handling() {
    let mut app = create_test_app_with_entries();

    // Test 'j' key (move down)
    let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key_event(key_event).unwrap();

    // Verify navigation state changed
    let initial_selection = 0;
    let new_selection = app.tree_view.list_state.selected().unwrap();
    assert_eq!(new_selection, initial_selection + 1);
}

#[test]
fn test_mode_switching_keys() {
    let mut app = create_test_app_with_entries();

    // Test Enter key (should enter edit mode)
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key_event(key_event).unwrap();

    assert!(matches!(app.mode, AppMode::Edit));

    // Test ESC key (should return to navigation)
    let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key_event(key_event).unwrap();

    assert!(matches!(app.mode, AppMode::Navigation));
}
```

## Property-Based Testing

### Using quickcheck for Advanced Testing

Add to Cargo.toml:

```toml
[dev-dependencies]
quickcheck = "1.0"
quickcheck_macros = "1.0"
```

```rust
use quickcheck_macros::quickcheck;

#[quickcheck]
fn test_date_parsing_roundtrip(year: u16, month: u8, day: u8) -> bool {
    // Only test valid dates
    if year < 1000 || year > 9999 || month == 0 || month > 12 || day == 0 || day > 31 {
        return true; // Skip invalid inputs
    }

    if let Ok(date) = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32) {
        let formatted = format_entry_date(&date);
        let parsed = parse_entry_date(&formatted);

        parsed.map(|p| p == date).unwrap_or(false)
    } else {
        true // Skip invalid dates
    }
}

#[quickcheck]
fn test_editor_operations_preserve_invariants(operations: Vec<EditorOperation>) -> bool {
    let mut editor = EditorState::new();

    for op in operations {
        apply_operation(&mut editor, op);

        // Check invariants
        if editor.cursor_line >= editor.lines.len() {
            return false;
        }

        if let Some(line) = editor.lines.get(editor.cursor_line) {
            if editor.cursor_col > line.len() {
                return false;
            }
        }
    }

    true
}

#[derive(Clone, Debug)]
enum EditorOperation {
    InsertChar(char),
    DeleteChar,
    MoveCursor(CursorDirection),
    InsertNewline,
}
```

## Performance Testing

### Basic Benchmarking

Add to Cargo.toml:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "tree_operations"
harness = false
```

```rust
// benches/tree_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engineering_journal::navigation::TreeViewState;

fn bench_tree_navigation(c: &mut Criterion) {
    let tree = create_large_tree(1000); // 1000 entries

    c.bench_function("tree navigation down", |b| {
        b.iter(|| {
            let mut tree_copy = tree.clone();
            for _ in 0..100 {
                tree_copy.move_down();
            }
            black_box(tree_copy);
        })
    });
}

fn bench_tree_building(c: &mut Criterion) {
    let entries = create_test_entries(1000);

    c.bench_function("build tree from entries", |b| {
        b.iter(|| {
            let tree = TreeViewState::from_entries(black_box(entries.clone()));
            black_box(tree);
        })
    });
}

criterion_group!(benches, bench_tree_navigation, bench_tree_building);
criterion_main!(benches);
```

## Test Data Management

### Test Fixtures

```rust
// src/test_utils.rs
pub mod test_utils {
    use super::*;
    use tempfile::TempDir;

    pub fn create_test_app() -> App {
        let temp_dir = TempDir::new().unwrap();
        App::new_with_storage_path(temp_dir.path()).unwrap()
    }

    pub fn create_test_entries(count: usize) -> Vec<Entry> {
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        (0..count)
            .map(|i| {
                let date = start_date + chrono::Duration::days(i as i64);
                let mut entry = Entry::new(date);
                entry.content = Some(format!("Test entry {}", i));
                entry
            })
            .collect()
    }

    pub fn create_test_tree_with_entries(entries: Vec<Entry>) -> TreeViewState {
        TreeViewState::from_entries(entries)
    }
}
```

## Continuous Integration

### GitHub Actions Configuration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run Clippy
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --verbose

      - name: Run integration tests
        run: cargo test --test integration_tests

      - name: Run benchmarks
        run: cargo bench --no-run
```

## Manual Testing Checklist

### Core Functionality

- [ ] App starts without crashes
- [ ] Tree navigation works (hjkl and arrows)
- [ ] Entry creation works (n and c keys)
- [ ] Entry editing works (Enter to edit, ESC to exit)
- [ ] Entry deletion works (d key with confirmation)
- [ ] File persistence works (entries saved and loaded correctly)

### Edge Cases

- [ ] Empty directory (no entries)
- [ ] Large number of entries (100+)
- [ ] Very long entry content
- [ ] Invalid file permissions
- [ ] Disk full scenarios
- [ ] Terminal resize handling

### User Experience

- [ ] Status bar updates correctly
- [ ] Key bindings feel responsive
- [ ] Error messages are helpful
- [ ] No flickering or visual artifacts

## Testing Best Practices

### 1. Test Organization

```rust
// Group related tests
mod entry_tests {
    use super::*;

    mod creation {
        use super::*;
        // Entry creation tests
    }

    mod persistence {
        use super::*;
        // File I/O tests
    }
}
```

### 2. Descriptive Test Names

```rust
#[test]
fn test_entry_creation_with_valid_date_creates_correct_file_path() {
    // Test implementation
}

#[test]
fn test_tree_navigation_wraps_around_when_reaching_end() {
    // Test implementation
}
```

### 3. Clear Assertions

```rust
#[test]
fn test_editor_cursor_movement() {
    let mut editor = EditorState::new();
    editor.insert_char('H');

    // Clear assertion with context
    assert_eq!(
        editor.cursor_col, 1,
        "Cursor should be at position 1 after inserting one character"
    );
}
```

Remember: Good tests are your safety net while learning Rust. They help you catch issues early and give you confidence to refactor and improve your code!
