# Terminal UI (TUI) Functional Requirements - MVP

## Overview

This document outlines the MVP functional requirements for the interactive terminal user interface for the `devlog` command-line tool. The TUI will be activated when users run `devlog list`, providing an interactive browsing experience for journal entries.

---

## Core Functional Requirements

### 1. Navigation

- **Arrow Keys** (`↑`, `↓`, `←`, `→`) - Navigate between entries and pages
- **Vim Keys** (`h`, `j`, `k`, `l`) - Alternative navigation (left, down, up, right)

### 2. Entry Actions

- **Enter** - View full content of selected entry
- **e** - Edit selected entry in $EDITOR
- **n** - Create new entry
- **q** - Quit application

### 3. Search and Help

- **/** or **s** - Search/filter entries (not-implemented placeholder for now)
- **h** or **?** - Show help with all keybindings

### 4. Pagination Support

- **Page Up/Down** or **Left/Right arrows** - Navigate between pages when there are many entries
- **Status bar** - Show current page and total entries (e.g., "Page 2/5 - Entry 15/47")

### 5. Entry Information Display

- **Entry list** - Show entry ID (date), first line of content, and basic metadata
- **Selection highlight** - Clear visual indication of currently selected entry

### 6. Error Handling

- **Confirmation prompts** - For destructive operations
- **Error messages** - Clear feedback when operations fail
- **Graceful fallback** - If TUI fails, fall back to simple list output

---

## Technical Implementation Notes

### TUI Framework

- Use `ratatui` (modern Rust TUI framework) with `crossterm` for terminal handling

### Entry Display Format

```
┌─ DevLog Entries ───────────────────────────────────────────┐
│ > 20240910  Fixed pagination bug in user service           │
│   20240909  Team meeting notes - Q4 planning               │
│   20240908  Implemented user authentication flow           │
│   20240907  Debugging database connection issues           │
└─ Page 1/3 - Entry 1/25 ──────── Press 'h' for help ────────┘
```

### Help Display

```
┌─ Help ─────────────────────────────────────────────────────┐
│ Navigation:                                                │
│   ↑/k - Up    ↓/j - Down    ←/h - Left    →/l - Right      │
│   Page Up/Down - Navigate pages                            │
│                                                            │
│ Actions:                                                   │
│   Enter - View entry    e - Edit    n - New    q - Quit    │
│   / or s - Search (coming soon)    h/? - Help              │
│                                                            │
│ Press any key to continue...                               │
└────────────────────────────────────────────────────────────┘
```
