use regex::Regex;

/// Result of parsing annotations from content
pub struct ParsedAnnotations {
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub tags: Vec<String>,
}

/// Extract annotations from text content
pub struct AnnotationParser {
    people_regex: Regex,
    projects_regex: Regex,
    tags_regex: Regex,
}

impl AnnotationParser {
    /// Create a new annotation parser with default patterns
    pub fn new() -> Self {
        Self {
            // Regex pattern: ([\w-]+): one or more word characters (letters/digits/underscore/hyphen)
            people_regex: Regex::new(r"@([\w-]+)").unwrap(),
            projects_regex: Regex::new(r"::([\w-]+)").unwrap(),
            tags_regex: Regex::new(r"\+([\w-]+)").unwrap(),
        }
    }

    /// Generic extraction function
    fn extract_with_regex(&self, content: &str, regex: &Regex) -> Vec<String> {
        regex
            .captures_iter(content)
            // capture[0] is the full match, i.e. @alice
            // capture[1] is the first capture group, i.e. (alice)
            // filter_map() returns the item that is `Some(value)`
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Extract all annotations from content
    /// @alice -> people
    /// ::search_engine -> projects
    /// +motivation -> tags
    pub fn parse(&self, content: &str) -> ParsedAnnotations {
        ParsedAnnotations {
            people: self.extract_with_regex(content, &self.people_regex),
            projects: self.extract_with_regex(content, &self.projects_regex),
            tags: (self.extract_with_regex(content, &self.tags_regex)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all_annotations() {
        let parser = AnnotationParser::new();
        let content =
            "@alice helped with ::project, then @alice worked on ::project using +rust and +rust";
        let annotations = parser.parse(content);

        // Should allow duplicates since we're using Vec instead of HashSet
        assert_eq!(annotations.people.len(), 2);
        assert_eq!(annotations.people, ["alice", "alice"].map(String::from));

        assert_eq!(annotations.projects.len(), 2);
        assert_eq!(
            annotations.projects,
            ["project", "project"].map(String::from)
        );

        assert_eq!(annotations.tags.len(), 2);
        assert_eq!(annotations.tags, ["rust", "rust"].map(String::from));
    }

    #[test]
    fn test_no_annotations() {
        let parser = AnnotationParser::new();
        let content = "Just worked alone today on some regular code";
        let annotations = parser.parse(content);

        assert!(annotations.people.is_empty());
        assert!(annotations.projects.is_empty());
        assert!(annotations.tags.is_empty());
    }

    #[test]
    fn test_ignore_incomplete_annotations() {
        let parser = AnnotationParser::new();
        let content = "The @ symbol alone or @ with space, :: without name, + without tag";
        let annotations = parser.parse(content);

        assert!(annotations.people.is_empty());
        assert!(annotations.projects.is_empty());
        assert!(annotations.tags.is_empty());
    }

    #[test]
    fn test_annotations_with_punctuation() {
        let parser = AnnotationParser::new();
        let content = "Talked to @alice! Then worked on ::project? Finally learned +rust.";
        let annotations = parser.parse(content);

        assert_eq!(annotations.people.len(), 1);
        assert_eq!(annotations.people, vec!["alice".to_string()]);

        assert_eq!(annotations.projects.len(), 1);
        assert_eq!(annotations.projects, vec!["project".to_string()]);

        assert_eq!(annotations.tags.len(), 1);
        assert_eq!(annotations.tags, vec!["rust".to_string()]);
    }

    #[test]
    fn test_multiline_content() {
        let parser = AnnotationParser::new();
        let content = r#"
        Day 1: Met with @sarah about ::search_engine
        Day 2: @mike joined, we used +rust
        Day 3: Deployed to ::production with +confidence
        "#;
        let annotations = parser.parse(content);

        assert_eq!(annotations.people.len(), 2);
        assert_eq!(
            annotations.people,
            vec!["sarah".to_string(), "mike".to_string()]
        );

        assert_eq!(annotations.projects.len(), 2);
        assert_eq!(
            annotations.projects,
            vec!["search_engine".to_string(), "production".to_string()]
        );

        assert_eq!(annotations.tags.len(), 2);
        assert_eq!(
            annotations.tags,
            vec!["rust".to_string(), "confidence".to_string()]
        );
    }

    #[test]
    fn test_duplicates_preserved() {
        let parser = AnnotationParser::new();
        let content =
            "@alice @bob @alice worked on ::proj1 ::proj2 ::proj1 using +rust +debug +rust";
        let annotations = parser.parse(content);

        // Duplicates should be preserved in order
        assert_eq!(annotations.people.len(), 3);
        assert_eq!(
            annotations.people,
            ["alice", "bob", "alice"].map(String::from)
        );

        assert_eq!(annotations.projects.len(), 3);
        assert_eq!(
            annotations.projects,
            ["proj1", "proj2", "proj1"].map(String::from)
        );

        assert_eq!(annotations.tags.len(), 3);
        assert_eq!(
            annotations.tags,
            ["rust", "debug", "rust"].map(String::from)
        );
    }
}
