# Functional Requirements

## 1. Core Entry Management

- **Create a new entry**

  - `devlog new` → open Vim editor for detailed entry
  - Support YAML frontmatter (created at, last updated at) + Markdown body

- **Edit an entry**

  - `devlog edit <id>` → open existing entry in Vim editor

- **List entries**

  - `devlog list` → show last 20 entries with navigation instructions
  - `devlog list --interactive` → launch TUI with tree navigation

- **Show entry**

  - `devlog show <id>` → display markdown-formatted entry content

---

## 2. Interactive TUI Mode

- **Two-panel interface** (`devlog list --interactive`):

  - **Left Panel**: Tree navigation (Year > Month > Date hierarchy)
  - **Right Panel**: Content display and editing

- **Navigation controls**:

  - `Tab` → switch between panels
  - `h`/`j` or `↑`/`↓` → navigate up/down
  - `k`/`l` or `←`/`→` → expand/collapse tree nodes
  - `Enter` → toggle nodes or show content
  - `/` → search for specific dates
  - `e` → edit content (launches Vim)

---

## 3. Insight & Analytics

- **Annotation parsing** (mechanical extraction):

  - `@alice` → people mentions
  - `::search-service` → project references
  - `+motivation` → tags/themes

- **Generate summary reports**:

  - `devlog insight` → extract and display projects, people, and tags
  - Basic statistics and collaboration insights

---

## 5. Sync & Storage

- **Local-first storage**

  - Entries stored in `~/.devlog/entries/YYYY-MM-DD.md`
  - Markdown + YAML frontmatter

- **Sync command** (premium/extension):

  - `devlog sync` → push to remote (e.g., Git repo, web dashboard, cloud backup).

---

## 6. Config & Customization

- **Config file** at `~/.devlog/config.yml`:

  - Default editor
  - Known coworkers, projects, tags
  - Sync settings

- **Templates** for entries (daily, weekly retro).

---

## 7. Extensibility Hooks (future)

- **IDE integration**: VS Code/JetBrains extension reusing CLI backend.
- **Slack/Discord integration**: `/devlog` command → append entry.
- **Git integration**: auto-suggest commits/issues for context.

---

✅ **MVP Focus** (Current Phase):

- Basic entry management (`new`, `edit`, `show`, `list`)
- Interactive TUI with tree navigation
- Simplified storage (stateless file scanning)
- Mechanical annotation parsing (`insight` command)
- Vim-based editing workflow
