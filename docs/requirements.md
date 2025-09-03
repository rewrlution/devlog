# Functional Requirements

## 1. Core Entry Management

- **Create a new entry**

  - `devlog new -m "message"` → inline note
  - `devlog new` → open \$EDITOR for detailed entry
  - Support YAML frontmatter (date, tags, people, projects) + Markdown body

- **Edit an entry**

  - `devlog edit <id>` → open existing entry in \$EDITOR

- **List entries**

  - `devlog list [--since <date>] [--until <date>] [--tag <tag>] [--project <project>] [--person <@name>]`

- **Show entry**

  - `devlog show <id>` → display entry details

---

## 2. Annotation & Metadata

- **Inline annotations (Markdown-compatible):**

  - `@alice` → coworker/person
  - `::search-service` → project
  - `+motivation` → tag/technology

- **Auto-extraction of metadata** into YAML frontmatter for easy querying.
- **Configurable dictionary** to autocomplete/validate coworkers, projects, tags.

---

## 3. Stats & Insights

- **Generate summary reports**:

  - Collaboration list (who you work with most)
  - Top projects mentioned
  - Sentiment/motivation trend over time
  - Frequency of blockers/issues

- CLI command: `devlog stats [--since <date>]`

---

## 4. Exporting

- **Brag doc export**

  - `devlog export --since last-quarter --format markdown`
  - Generates bullet points grouped by project, theme, or outcome

- Support export formats: Markdown, CSV (others later: PDF, HTML).

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

✅ **MVP focus** (phase 1):

- Entry creation (`new`, `list`, `show`, `edit`)
- Annotation parsing (`@`, `::`, `+`)
- Local Markdown storage with YAML frontmatter
- Basic `stats` (counts, top collaborators/projects)
- Basic `export` (Markdown brag doc)
