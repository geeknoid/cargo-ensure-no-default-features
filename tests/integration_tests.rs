//! Integration tests for cargo-ensure-no-default-features
//!
//! These tests validate the tool's behavior by running it against
//! various test Cargo.toml configurations.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Helper to get the path to the compiled cargo-ensure-no-default-features binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("cargo-ensure-no-default-features");
    let _ = path.set_extension(std::env::consts::EXE_EXTENSION);
    path
}

/// Helper to create a temporary test directory with a Cargo.toml file
fn create_test_manifest(content: &str) -> tempfile::TempDir {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("Cargo.toml");
    fs::write(&manifest_path, content).expect("Failed to write test Cargo.toml");
    temp_dir
}

#[test]
fn test_valid_workspace_all_deps_have_default_features_false() {
    let content = r#"
[workspace]
members = ["crate1", "crate2"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false }
tokio = { version = "1.0", default-features = false, features = ["rt"] }
anyhow = { version = "1.0", default-features = false }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ All workspace dependencies have default-features = false"));
}

#[test]
fn test_invalid_simple_version_string() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = "1.0"
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'serde'"));
    assert!(stderr.contains("uses simple version string"));
}

#[test]
fn test_invalid_missing_default_features() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0" }
tokio = { version = "1.0", features = ["rt"] }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'serde'"));
    assert!(stderr.contains("missing default-features = false"));
    assert!(stderr.contains("'tokio'"));
}

#[test]
fn test_invalid_default_features_true() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0", default-features = true }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("'serde'"));
    assert!(stderr.contains("has default-features = true"));
}

#[test]
fn test_mixed_valid_and_invalid_dependencies() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
# Valid
anyhow = { version = "1.0", default-features = false }
tokio = { version = "1.0", default-features = false, features = ["rt"] }

# Invalid - simple string
serde = "1.0"

# Invalid - missing default-features
regex = { version = "1.0" }

# Invalid - default-features = true
clap = { version = "4.0", default-features = true }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should report all 3 errors
    assert!(stderr.contains("'serde'"));
    assert!(stderr.contains("'regex'"));
    assert!(stderr.contains("'clap'"));
    assert!(stderr.contains("Found 3 dependency validation error(s)"));
}

#[test]
fn test_no_workspace_section() {
    let content = r#"
[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No [workspace] section found"));
}

#[test]
fn test_no_workspace_dependencies_section() {
    let content = r#"
[workspace]
members = ["crate1"]
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No [workspace.dependencies] section found"));
}

#[test]
fn test_empty_workspace_dependencies() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with empty dependencies");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ All workspace dependencies have default-features = false"));
}

#[test]
fn test_nonexistent_manifest_path() {
    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg("nonexistent-file.toml")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to read"));
}

#[test]
fn test_invalid_toml_syntax() {
    let content = "
[workspace
this is not valid toml
";

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Command should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to parse"));
}

#[test]
fn test_dependency_with_git_source() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
my-dep = { git = "https://github.com/example/repo", default-features = false }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with git dependency");
}

#[test]
fn test_dependency_with_path_source() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
my-local-dep = { path = "../other-crate", default-features = false }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with path dependency");
}

#[test]
fn test_dependency_with_multiple_features() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
tokio = { version = "1.0", default-features = false, features = ["rt", "macros", "sync", "time"] }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with multiple features");
}

#[test]
fn test_default_manifest_path() {
    // Create a Cargo.toml in a temp directory and run from that directory
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false }
"#;

    let temp_dir = create_test_manifest(content);

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with default manifest path");
}

#[test]
fn test_dependency_with_optional_flag() {
    let content = r#"
[workspace]
members = ["crate1"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false, optional = true }
"#;

    let temp_dir = create_test_manifest(content);
    let manifest_path = temp_dir.path().join("Cargo.toml");

    let output = Command::new(get_binary_path())
        .arg("ensure-no-default-features")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed with optional dependency");
}
