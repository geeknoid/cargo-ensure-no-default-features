use anyhow::{Context, Result};

/// Validates a single dependency entry and returns an error message if invalid.
fn validate_dependency(name: &str, value: &toml::Value) -> Result<(), String> {
    if value.is_str() {
        return Err(format!(
            "  - '{name}': uses simple version string, should be a table with default-features = false",
        ));
    }

    let Some(dep_table) = value.as_table() else {
        return Err(format!("  - '{name}': dependency is not a table"));
    };

    match dep_table.get("default-features") {
        Some(toml::Value::Boolean(false)) => Ok(()),

        Some(toml::Value::Boolean(true)) => Err(format!("  - '{name}': has default-features = true (must be false)")),

        None => Err(format!("  - '{name}': missing default-features = false")),

        Some(_) => Err(format!(
            "  - '{name}': default-features has unexpected value (must be boolean false)",
        )),
    }
}

/// Validates all workspace dependencies in the given Cargo.toml content
///
/// # Returns
///
/// A tuple containing:
/// * A vector of error messages for invalid dependencies
/// * A vector of all dependency names found in [workspace.dependencies]
pub fn validate_workspace_dependencies(content: &str, exceptions: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    let parsed: toml::Value = toml::from_str(content).context("Failed to parse Cargo.toml")?;
    let workspace = parsed.get("workspace").context("No [workspace] section found")?;
    let dependencies = workspace.get("dependencies").context("No [workspace.dependencies] section found")?;
    let deps_table = dependencies.as_table().context("[workspace.dependencies] is not a table")?;

    let mut errors = Vec::new();
    let mut found_deps = Vec::new();
    for (name, value) in deps_table {
        found_deps.push(name.clone());
        if exceptions.contains(name) {
            continue;
        }

        if let Err(err) = validate_dependency(name, value) {
            errors.push(err);
        }
    }

    Ok((errors, found_deps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dependency_with_default_features_false() {
        let toml_str = r#"
version = "1.0"
default-features = false
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid when default-features = false");
    }

    #[test]
    fn test_validate_dependency_with_default_features_false_and_features() {
        let toml_str = r#"
version = "1.0"
default-features = false
features = ["feature1", "feature2"]
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid with default-features = false and features");
    }

    #[test]
    fn test_validate_dependency_simple_version_string() {
        let value = toml::Value::String("1.0".to_string());

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("test-crate"));
        assert!(error.contains("uses simple version string"));
    }

    #[test]
    fn test_validate_dependency_not_a_table() {
        // Test with an array value (not a string or table)
        let value = toml::Value::Array(vec![toml::Value::String("1.0".to_string())]);

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("test-crate"));
        assert!(error.contains("dependency is not a table"));
    }

    #[test]
    fn test_validate_dependency_missing_default_features() {
        let toml_str = r#"
version = "1.0"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("test-crate"));
        assert!(error.contains("missing default-features = false"));
    }

    #[test]
    fn test_validate_dependency_default_features_true() {
        let toml_str = r#"
version = "1.0"
default-features = true
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("test-crate"));
        assert!(error.contains("has default-features = true"));
    }

    #[test]
    fn test_validate_dependency_with_git_source() {
        let toml_str = r#"
git = "https://github.com/example/repo"
default-features = false
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid with git source and default-features = false");
    }

    #[test]
    fn test_validate_dependency_with_path_source() {
        let toml_str = r#"
path = "../local-crate"
default-features = false
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid with path source and default-features = false");
    }

    #[test]
    fn test_validate_dependency_with_optional_flag() {
        let toml_str = r#"
version = "1.0"
default-features = false
optional = true
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid with optional flag and default-features = false");
    }

    #[test]
    fn test_validate_dependency_default_features_string() {
        let toml_str = r#"
version = "1.0"
default-features = "false"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("test-crate"));
        assert!(error.contains("unexpected value"));
    }

    #[test]
    fn test_validate_dependency_complex_configuration() {
        let toml_str = r#"
version = "1.0"
default-features = false
features = ["feat1", "feat2"]
optional = true
package = "other-name"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_ok(), "Should be valid with complex configuration");
    }

    #[test]
    fn test_validate_dependency_git_without_default_features() {
        let toml_str = r#"
git = "https://github.com/example/repo"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("missing default-features = false"));
    }

    #[test]
    fn test_validate_dependency_path_without_default_features() {
        let toml_str = r#"
path = "../local-crate"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();

        let result = validate_dependency("test-crate", &value);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("missing default-features = false"));
    }

    #[test]
    fn test_validate_workspace_dependencies_all_valid() {
        let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false }
tokio = { version = "1.0", default-features = false, features = ["rt"] }
"#;

        let errors = validate_workspace_dependencies(content, &[]).unwrap();
        assert!(errors.0.is_empty(), "Should have no errors with all valid dependencies");
    }

    #[test]
    fn test_validate_workspace_dependencies_with_errors() {
        let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = "1.0"
tokio = { version = "1.0" }
"#;

        let errors = validate_workspace_dependencies(content, &[]).unwrap();
        assert_eq!(errors.0.len(), 2, "Should have 2 errors");
    }

    #[test]
    fn test_validate_workspace_dependencies_no_workspace() {
        let content = r#"
[package]
name = "test"
version = "0.1.0"
"#;

        let result = validate_workspace_dependencies(content, &[]);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("No [workspace] section found"));
    }

    #[test]
    fn test_validate_workspace_dependencies_no_dependencies() {
        let content = r#"
[workspace]
members = ["crate1"]
"#;

        let result = validate_workspace_dependencies(content, &[]);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("No [workspace.dependencies] section found"));
    }

    #[test]
    fn test_validate_workspace_dependencies_empty_dependencies() {
        let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
"#;

        let errors = validate_workspace_dependencies(content, &[]).unwrap();
        assert!(errors.0.is_empty(), "Should have no errors with empty dependencies");
    }

    #[test]
    fn test_validate_workspace_dependencies_with_exceptions() {
        let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false }
tokio = { version = "1.0", default-features = false, features = ["rt"] }
"#;

        let exceptions = vec!["tokio".to_string()];
        let errors = validate_workspace_dependencies(content, &exceptions).unwrap();
        assert!(errors.0.is_empty(), "Should have no errors with valid dependencies");
        assert_eq!(errors.1.len(), 2, "Should find 2 dependencies");
        assert!(errors.1.contains(&"serde".to_string()));
        assert!(errors.1.contains(&"tokio".to_string()));
    }

    #[test]
    fn test_validate_workspace_dependencies_with_exceptions_and_errors() {
        let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = "1.0"
tokio = { version = "1.0" }
"#;

        let exceptions = vec!["tokio".to_string()];
        let errors = validate_workspace_dependencies(content, &exceptions).unwrap();
        assert_eq!(errors.0.len(), 1, "Should have 1 error");
        assert_eq!(errors.1.len(), 2, "Should find 2 dependencies");
        assert!(errors.1.contains(&"serde".to_string()));
        assert!(errors.1.contains(&"tokio".to_string()));
    }
}
