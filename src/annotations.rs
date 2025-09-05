use regex::Regex;
use std::collections::HashSet;

/// Result of parsing annotations from content
pub struct ParsedAnnotations {
    pub people: HashSet<String>,
    pub projects: HashSet<String>,
    pub tags: HashSet<String>,
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
    fn extract_with_regex(&self, content: &str, regex: &Regex) -> HashSet<String> {
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

        // Should deduplicate automatically due to HashSet
        assert_eq!(annotations.people.len(), 1);
        assert!(annotations.people.contains("alice"));

        assert_eq!(annotations.projects.len(), 1);
        assert!(annotations.projects.contains("project"));

        assert_eq!(annotations.tags.len(), 1);
        assert!(annotations.tags.contains("rust"));
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
        assert!(annotations.people.contains("alice"));

        assert_eq!(annotations.projects.len(), 1);
        assert!(annotations.projects.contains("project"));

        assert_eq!(annotations.tags.len(), 1);
        assert!(annotations.tags.contains("rust"));
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
        assert!(annotations.people.contains("sarah"));
        assert!(annotations.people.contains("mike"));

        assert_eq!(annotations.projects.len(), 2);
        assert!(annotations.projects.contains("search_engine"));
        assert!(annotations.projects.contains("production"));

        assert_eq!(annotations.tags.len(), 2);
        assert!(annotations.tags.contains("rust"));
        assert!(annotations.tags.contains("confidence"));
    }
}
