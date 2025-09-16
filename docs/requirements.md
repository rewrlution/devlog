# Engineering Journal App - MVP Functional Requirements

## Project Overview

A terminal-based journal keeping application designed specifically for engineers, built with Rust and `ratatui`. The app provides a vim-like navigation experience with efficient keyboard shortcuts for managing daily journal entries.

## Core Functionality (MVP)

### 1. Entry Management (CRUD Operations)

#### Create Entry

- **Entry ID Format**: `YYYYMMDD` (e.g., `20250915`)
- **Auto-creation**: Automatically create today's entry if it doesn't exist
- **Manual creation**: Allow creating entries for specific dates

#### Read/View Entry

- Display entry content in a dedicated view pane
- Show entry in plain text format

#### Update Entry

- Modify existing entry content using built-in editor
- Save changes when user explicitly saves (Ctrl+S or :w)

#### Delete Entry

- Simple delete with confirmation prompt

### 2. Navigation System

#### Hierarchical Navigation Structure

```
Year (2025)
├── Month (Jan, Feb, Mar...)
│   ├── Day (01, 02, 03...)
│   │   └── Entry Content
```

#### Keyboard Navigation (Vim-inspired + Arrow Keys)

```
Movement (Vim-style):
- h / ←: Move left (collapse/go to parent level)
- j / ↓: Move down (next item in current level)
- k / ↑: Move up (previous item in current level)
- l / →: Move right (expand/go to child level)

Entry Management:
- Enter: Open/Edit selected entry (creates if doesn't exist)
- n: Create new entry for today
- c: Create entry for specific date (prompts for YYYYMMDD)
- d: Delete selected entry (with confirmation)
- Space: Toggle expand/collapse

Mode & Application:
- ESC: Return to navigation mode (from edit mode)
- Ctrl+S: Save entry (in edit mode)
- q: Quit application
```

#### Smart Navigation Features

- **Year View**: Navigate between years (2023, 2024, 2025...)
- **Month View**: Navigate months within a year (Jan, Feb, Mar...)
- **Day View**: Navigate days within a month (01, 02, 03...)
- **Breadcrumb Navigation**: Always show current location (2025 > Mar > 15)

### 3. User Interface Design

#### Main Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Engineering Journal v1.0                           [h] Help │
├─────────────────────────────────────────────────────────────┤
│ Navigation: 2025 > March > 15                               │
├─────────────────┬───────────────────────────────────────────┤
│ Tree View       │ Entry Content                             │
│                 │                                           │
│ ▼ 2025          │ # March 15, 2025                          │
│   ▼ March       │                                           │
│     • 01        │ ## Daily Standup                          │
│     • 03        │ - Fixed authentication bug                │
│     • 07        │ - Reviewed PR #123                        │
│     ● 15        │                                           │
│     • 28        │ ## Technical Notes                        │
│   ▼ April       │ Discovered interesting pattern in...      │
│     • 02        │                                           │
│     • 05        │                                           │
│   ▶ May         │                                           │
│                 │                                           │
│                 │                                           │
├─────────────────┴───────────────────────────────────────────┤
│ [hjkl/↑↓←→] Nav [Enter] Edit [n] New [c] Create [d] Del [q] Quit │
└─────────────────────────────────────────────────────────────┘
```

#### Visual Indicators

- `▼` Expanded folder (year/month)
- `▶` Collapsed folder
- `●` Current/selected entry
- `•` Available entry

### 3. Built-in Editor

#### Editor Modes

- **Navigation Mode**: Browse entries using hjkl/arrow key navigation
- **Edit Mode**: Edit entry content with basic text editing capabilities

#### Editor Key Bindings

```
Navigation Mode:
- hjkl / ↑↓←→: Navigate tree
- Enter: Open entry for editing (creates if doesn't exist)
- n: Create new entry for today
- c: Create entry for specific date (prompts for YYYYMMDD)
- d: Delete selected entry (with confirmation)
- Space: Toggle expand/collapse
- q: Quit application

Edit Mode:
- ESC: Return to navigation mode
- Ctrl+S: Save entry
- Basic text editing (insert, delete, arrow keys for cursor movement)
- Ctrl+L: Clear screen/refresh
```

#### Editor Features

- Simple text editing capabilities
- Line-based editing
- Basic cursor movement with arrow keys
- Insert and delete text

### 4. Data Storage

#### File Structure

```
~/.config/engineering-journal/
├── entries/
│   ├── 2025/
│   │   ├── 01/
│   │   │   ├── 20250101.md
│   │   │   ├── 20250102.md
│   │   │   └── ...
│   │   ├── 02/
│   │   └── ...
│   └── 2024/
```

#### Entry Format

- **File Format**: Plain text (`.txt`) or Markdown (`.md`)
- **Naming Convention**: `YYYYMMDD.md`
- **Content**: Simple text content without metadata

### 5. Common Workflows

#### Creating Entries

**Scenario 1: Create today's entry**

1. Press `n` from anywhere in navigation mode
2. App creates entry for current date and opens editor
3. User writes content and saves with `Ctrl+S`

**Scenario 2: Create entry for specific date**

1. Press `c` from anywhere in navigation mode
2. App prompts for date input (YYYYMMDD format)
3. App creates entry for specified date and opens editor
4. User writes content and saves with `Ctrl+S`

#### Deleting Entries

**Delete workflow**

1. Navigate to existing entry
2. Press `d` to delete
3. App shows confirmation prompt: "Delete entry for YYYYMMDD? (y/N)"
4. Press `y` to confirm or `n`/`ESC` to cancel
5. If confirmed, entry is deleted and removed from tree view

#### Editing Existing Entries

**Edit workflow**

1. Navigate to existing entry using hjkl/arrows
2. Press `Enter` to open editor
3. Modify content using basic text editing
4. Save with `Ctrl+S` and return to navigation with `ESC`

## Implementation Plan

### MVP Features (Phase 1)

1. Basic file structure creation and management
2. Hierarchical navigation with hjkl/arrow key support
3. Entry creation with YYYYMMDD format
4. Simple built-in text editor with two modes
5. Save/load entries from filesystem
6. Basic UI layout with tree view and content pane

### Key Implementation Focus

- **Navigation**: Smooth hjkl + arrow key navigation between years/months/days
- **Editor**: Simple two-mode system (navigation vs edit)
- **File I/O**: Basic read/write operations for entry files
- **UI**: Clean split-pane layout with ratatui widgets

### Success Criteria

- **Usability**: Engineers can efficiently create and navigate entries using familiar key bindings
- **Performance**: Fast navigation between entries without lag
- **Simplicity**: Intuitive interface that doesn't require documentation to use
