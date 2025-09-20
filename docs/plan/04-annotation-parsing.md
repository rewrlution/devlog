# Step 4: Annotation Parsing and Insights

## Overview

Implement mechanical parsing of annotations (`@people`, `::projects`, `+tags`) and the `insight` command to generate summary reports.

## Annotation Parser (`src/utils/parser.rs`)

```rust
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Annotations {
    pub people: HashSet<String>,
    pub projects: HashSet<String>,
    pub tags: HashSet<String>,
}

impl Annotations {
    pub fn new() -> Self {
        Self {
            people: HashSet::new(),
            projects: HashSet::new(),
            tags: HashSet::new(),
        }
    }

    pub fn merge(&mut self, other: Annotations) {
        self.people.extend(other.people);
        self.projects.extend(other.projects);
        self.tags.extend(other.tags);
    }
}

pub fn parse_annotations(content: &str) -> Annotations {
    let mut annotations = Annotations::new();

    // Parse @people mentions
    let people_regex = Regex::new(r"@([a-zA-Z0-9_-]+)").unwrap();
    for cap in people_regex.captures_iter(content) {
        if let Some(person) = cap.get(1) {
            annotations.people.insert(person.as_str().to_string());
        }
    }

    // Parse ::project references
    let project_regex = Regex::new(r"::([a-zA-Z0-9_-]+)").unwrap();
    for cap in project_regex.captures_iter(content) {
        if let Some(project) = cap.get(1) {
            annotations.projects.insert(project.as_str().to_string());
        }
    }

    // Parse +tags
    let tag_regex = Regex::new(r"\+([a-zA-Z0-9_-]+)").unwrap();
    for cap in tag_regex.captures_iter(content) {
        if let Some(tag) = cap.get(1) {
            annotations.tags.insert(tag.as_str().to_string());
        }
    }

    annotations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_annotations() {
        let content = r#"
        Had a meeting with @alice and @bob about ::search-service.
        Working on +rust +cli implementation.
        Also discussed ::auth-system with @charlie.
        Added +productivity and +tools tags.
        "#;

        let annotations = parse_annotations(content);

        assert_eq!(annotations.people.len(), 3);
        assert!(annotations.people.contains("alice"));
        assert!(annotations.people.contains("bob"));
        assert!(annotations.people.contains("charlie"));

        assert_eq!(annotations.projects.len(), 2);
        assert!(annotations.projects.contains("search-service"));
        assert!(annotations.projects.contains("auth-system"));

        assert_eq!(annotations.tags.len(), 4);
        assert!(annotations.tags.contains("rust"));
        assert!(annotations.tags.contains("cli"));
        assert!(annotations.tags.contains("productivity"));
        assert!(annotations.tags.contains("tools"));
    }
}
```

## Insights Command (`src/commands/insight.rs`)

```rust
use crate::storage::Storage;
use crate::utils::parser::{parse_annotations, Annotations};
use color_eyre::Result;
use std::collections::HashMap;

pub fn execute() -> Result<()> {
    let storage = Storage::new()?;
    let entry_ids = storage.list_entries()?;

    if entry_ids.is_empty() {
        println!("No entries found. Create some entries first with 'devlog new'");
        return Ok(());
    }

    println!("Analyzing {} entries...", entry_ids.len());
    println!();

    let mut all_annotations = Annotations::new();
    let mut entry_count = 0;
    let mut people_frequency = HashMap::new();
    let mut project_frequency = HashMap::new();
    let mut tag_frequency = HashMap::new();

    // Process all entries
    for entry_id in entry_ids {
        if let Ok(entry) = storage.load_entry(&entry_id) {
            let annotations = parse_annotations(&entry.content);

            // Count frequencies
            for person in &annotations.people {
                *people_frequency.entry(person.clone()).or_insert(0) += 1;
            }
            for project in &annotations.projects {
                *project_frequency.entry(project.clone()).or_insert(0) += 1;
            }
            for tag in &annotations.tags {
                *tag_frequency.entry(tag.clone()).or_insert(0) += 1;
            }

            all_annotations.merge(annotations);
            entry_count += 1;
        }
    }

    // Display insights
    display_insights(&all_annotations, entry_count, &people_frequency, &project_frequency, &tag_frequency);

    Ok(())
}

fn display_insights(
    annotations: &Annotations,
    entry_count: usize,
    people_freq: &HashMap<String, usize>,
    project_freq: &HashMap<String, usize>,
    tag_freq: &HashMap<String, usize>,
) {
    println!("📊 Development Log Insights");
    println!("=" .repeat(40));
    println!();

    // Basic stats
    println!("📈 Statistics:");
    println!("  Total entries analyzed: {}", entry_count);
    println!("  Unique people mentioned: {}", annotations.people.len());
    println!("  Unique projects referenced: {}", annotations.projects.len());
    println!("  Unique tags used: {}", annotations.tags.len());
    println!();

    // Top people
    if !people_freq.is_empty() {
        println!("👥 People (most mentioned):");
        let mut people_sorted: Vec<_> = people_freq.iter().collect();
        people_sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (person, count) in people_sorted.iter().take(10) {
            println!("  @{:<15} ({})", person, count);
        }
        println!();
    }

    // Top projects
    if !project_freq.is_empty() {
        println!("🚀 Projects (most referenced):");
        let mut projects_sorted: Vec<_> = project_freq.iter().collect();
        projects_sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (project, count) in projects_sorted.iter().take(10) {
            println!("  ::{:<15} ({})", project, count);
        }
        println!();
    }

    // Top tags
    if !tag_freq.is_empty() {
        println!("🏷️  Tags (most used):");
        let mut tags_sorted: Vec<_> = tag_freq.iter().collect();
        tags_sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (tag, count) in tags_sorted.iter().take(10) {
            println!("  +{:<15} ({})", tag, count);
        }
        println!();
    }

    // Collaboration insights
    if annotations.people.len() > 1 {
        println!("🤝 Collaboration Insights:");
        println!("  You frequently collaborate with {} people", annotations.people.len());
        if let Some((top_person, count)) = people_freq.iter().max_by_key(|(_, v)| *v) {
            println!("  Most frequent collaborator: @{} ({} mentions)", top_person, count);
        }
        println!();
    }

    // Project insights
    if annotations.projects.len() > 0 {
        println!("💼 Project Insights:");
        println!("  You're working on {} different projects", annotations.projects.len());
        if let Some((top_project, count)) = project_freq.iter().max_by_key(|(_, v)| *v) {
            println!("  Most active project: ::{} ({} mentions)", top_project, count);
        }
        println!();
    }

    // Suggestions
    println!("💡 Suggestions:");
    if annotations.people.is_empty() {
        println!("  • Consider mentioning collaborators with @username");
    }
    if annotations.projects.is_empty() {
        println!("  • Tag projects with ::project-name for better tracking");
    }
    if annotations.tags.is_empty() {
        println!("  • Use +tags to categorize your work");
    }
    if annotations.people.len() > 0 && annotations.projects.len() > 0 {
        println!("  • Great job using annotations! Your logs are well-structured.");
    }
}
```

## Enhanced Entry Display

Update `src/commands/show.rs` to highlight annotations:

```rust
use crate::storage::Storage;
use crate::utils::parser::parse_annotations;
use color_eyre::{eyre::Context, Result};

pub fn execute(id: String) -> Result<()> {
    let storage = Storage::new()?;

    let entry = storage.load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    // Display entry with metadata
    println!("# Entry: {}", entry.id);
    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));

    // Parse and display annotations
    let annotations = parse_annotations(&entry.content);
    if !annotations.people.is_empty() || !annotations.projects.is_empty() || !annotations.tags.is_empty() {
        println!();
        println!("Annotations found:");
        if !annotations.people.is_empty() {
            let people: Vec<_> = annotations.people.iter().collect();
            println!("  People: {}", people.join(", "));
        }
        if !annotations.projects.is_empty() {
            let projects: Vec<_> = annotations.projects.iter().collect();
            println!("  Projects: {}", projects.join(", "));
        }
        if !annotations.tags.is_empty() {
            let tags: Vec<_> = annotations.tags.iter().collect();
            println!("  Tags: {}", tags.join(", "));
        }
    }

    println!("---");
    println!("{}", entry.content);

    Ok(())
}
```

## Testing Framework

Add to `Cargo.toml` for testing:

```toml
[dev-dependencies]
# No external test dependencies needed - Rust has built-in testing
```

## Key Rust Concepts Explained

- **HashSet**: Like a Set in other languages - stores unique values
- **HashMap**: Like a dictionary/map - stores key-value pairs
- **Regex**: Pattern matching for text (like in other languages)
- **#[cfg(test)]**: Rust's way of marking test-only code
- **unwrap()**: "I'm sure this won't fail" - use carefully
- **extend()**: Adds all items from one collection to another

## Implementation Tasks

1. **Add regex to Cargo.toml** (already done in Step 1)
2. **Create `src/utils/parser.rs`** with annotation parsing
3. **Create `src/commands/insight.rs`** with insights generation
4. **Update `src/commands/show.rs`** to display annotations
5. **Run tests** with `cargo test` to verify parsing works

## Testing Your Implementation

```bash
# Create an entry with annotations
cargo run new
# In vim, add content like:
# Met with @alice about ::search-service. Used +rust +productivity

# Generate insights
cargo run insight

# View entry with annotations
cargo run show 2024-09-20
```

## Example Output

```
📊 Development Log Insights
========================================

📈 Statistics:
  Total entries analyzed: 5
  Unique people mentioned: 3
  Unique projects referenced: 2
  Unique tags used: 4

👥 People (most mentioned):
  @alice          (3)
  @bob            (2)
  @charlie        (1)

🚀 Projects (most referenced):
  ::search-service (4)
  ::auth-system   (2)

🏷️  Tags (most used):
  +rust           (5)
  +productivity   (3)
  +cli            (2)
  +tools          (1)
```

## Next Steps

Move to Step 5: Interactive TUI to implement the terminal user interface with tree navigation.
