use std::path::PathBuf;

pub const DEFAULT_BASE_PATH: &str = "~/.devlog";
pub const DEFAULT_AZURE_CONTAINER: &str = "devlog";

/// Validate and normalize a base path
pub fn validate_base_path(path: &str) -> color_eyre::Result<PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return Err(color_eyre::eyre::eyre!("Base path cannot be empty"));
    }
    
    Ok(PathBuf::from(path))
}

/// Validate container name
pub fn validate_container_name(name: &str) -> color_eyre::Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(color_eyre::eyre::eyre!("Container name cannot be empty"));
    }
    
    // Azure container name validation rules
    if name.len() < 3 || name.len() > 63 {
        return Err(color_eyre::eyre::eyre!(
            "Container name must be between 3 and 63 characters"
        ));
    }
    
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(color_eyre::eyre::eyre!(
            "Container name can only contain lowercase letters, numbers, and hyphens"
        ));
    }
    
    if name.starts_with('-') || name.ends_with('-') {
        return Err(color_eyre::eyre::eyre!(
            "Container name cannot start or end with a hyphen"
        ));
    }
    
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_base_path() {
        assert!(validate_base_path("~/.devlog").is_ok());
        assert!(validate_base_path("/home/user/logs").is_ok());
        assert!(validate_base_path("").is_err());
        assert!(validate_base_path("   ").is_err());
    }

    #[test]
    fn test_validate_container_name() {
        assert!(validate_container_name("devlog").is_ok());
        assert!(validate_container_name("my-devlog-123").is_ok());
        
        assert!(validate_container_name("").is_err());
        assert!(validate_container_name("ab").is_err()); // too short
        assert!(validate_container_name("MyContainer").is_err()); // uppercase
        assert!(validate_container_name("-devlog").is_err()); // starts with hyphen
        assert!(validate_container_name("devlog-").is_err()); // ends with hyphen
    }
}