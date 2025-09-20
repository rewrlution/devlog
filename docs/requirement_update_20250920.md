# Functional Requirements Update - September 2025

After exploring various implementation approaches, I've decided to make significant changes to the basic functional requirements.

## 1. Storage Architecture

**Remove the event sourcing implementation** - it's unnecessarily complex for our use case.
**Remove entity extraction** - this will be discussed later.

**New simplified approach:**

- Scan the `.devlog/entries` folder and return all file names (`ls -la`). This should perform well even with 1000+ files, which is equivalent of ~3 years of entries.
- Only read file content when user runs the `show` command.
- Make the file content stateless. This means we will need to parse the frontmatter and update the `LastUpdatedAt` timestamp when user makes changes to the file.

## 2. Command Line Interface

**Commands:**

- `new`/`edit` - Launch Vim editor for creating/editing entries
- `show` - Display markdown-formatted content of a specific file
- `list` - Show the last 20 entries with a message: "To view additional entries, use `show --id YYYYMMDD` or launch interactive mode with `list --interactive`"

## 3. Interactive Mode (`list --interactive`)

**TUI Application with Two Panels:**

### Left Panel: Tree Navigation

- **Parent nodes:** Year and month
- **Child nodes:** Individual dates

**Navigation controls:**

- `Tab` - Switch focus between panels (navigation and content display panel)
- `h`/`j` or `↑`/`↓` arrows - Navigate up and down
- `k`/`l` or `←`/`→` arrows - Expand and collapse parent nodes
- `Enter` - Toggle parent nodes or show content for child nodes
- `/` - Search for specific dates (does not support content search at the moment)

### Right Panel: Content Display

When focused on the content panel:

- `h`/`j` or `↑`/`↓` arrows - Scroll through content
- `e` - Edit content (launches Vim)
- `:wq` - Exit Vim and return to content panel

## 4. Insight Command

**Purpose:** Analyze all files to extract structured information

**Implementation:**

- Scan all files and run annotation parser
- Extract projects, people, and tags
- Display results in organized format
- **Current scope:** Mechanical parsing only (AI support planned for future)
- **Note:** Performance and accuracy improvements needed
