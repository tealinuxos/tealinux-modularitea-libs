//! TOML Profile Loader
//!
//! Loads profile configurations from TOML files.

use crate::domain::Profile;
use crate::error::{ModulariteaError, Result};
use std::fs;
use std::path::Path;

/// Loader for TOML profile files
pub struct TomlLoader;

impl TomlLoader {
    /// Load a profile from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Profile> {
        let path = path.as_ref();

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| ModulariteaError::ProfileReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Parse TOML
        let profile: Profile =
            toml::from_str(&content).map_err(|e| ModulariteaError::ProfileParseError {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Validate basic constraints
        Self::validate(&profile)?;

        Ok(profile)
    }

    /// Load a profile from a string (useful for testing)
    pub fn load_from_string(content: &str) -> Result<Profile> {
        let profile: Profile =
            toml::from_str(content).map_err(|e| ModulariteaError::ProfileParseError {
                path: Path::new("memory").to_path_buf(),
                source: e,
            })?;

        Self::validate(&profile)?;

        Ok(profile)
    }

    /// Basic validation of the profile
    fn validate(profile: &Profile) -> Result<()> {
        if profile.meta.name.is_empty() {
            return Err(ModulariteaError::ProfileValidationError {
                message: "Profile name cannot be empty".to_string(),
            });
        }

        // Add more validation logic here if needed

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_profile() {
        let toml = r#"
            [meta]
            name = "test-profile"
            description = "A test profile"
            version = "0.1.0"
            
            [packages]
            install = ["vim", "git"]
        "#;

        let profile = TomlLoader::load_from_string(toml).unwrap();
        assert_eq!(profile.meta.name, "test-profile");
        assert_eq!(profile.packages.install.len(), 2);
    }

    #[test]
    fn test_missing_name_error() {
        let toml = r#"
            [meta]
            description = "Missing name"
        "#;

        let result = TomlLoader::load_from_string(toml);
        assert!(result.is_err());
        match result {
            Err(ModulariteaError::ProfileParseError { .. }) => {} /* Expected since name is required in struct but let's see serde behavior or validate */
            Err(ModulariteaError::ProfileValidationError { .. }) => {}
            _ => panic!("Unexpected error type"),
        }
    }
}
