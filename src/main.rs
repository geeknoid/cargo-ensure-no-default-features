#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! Eliminate superfluous features in a Rust workspace.
//!
//! This tool checks that all workspace dependencies in Cargo.toml have
//! `default-features = false`. This is a best practice in repos that publish multiple independent
//! crates, to ensure that each individual crate has the minimal set of features they need.
//! This can improve build times for any consumers of these crates by avoiding
//! unnecessary features being enabled by default.
//!
//! Install with:
//!
//! ```bash
//! cargo install cargo-ensure-no-default-features
//! ```
//!
//! And use with:
//!
//! ```bash
//! cargo ensure-no-default-features
//! ```
//!
//! The --manifest-path option lets you specify an explicit Cargo.toml file to check. Without this
//! option, it defaults to the Cargo.toml in the current directory.
//!
//! The --exceptions option lets you specify a comma-separated list of dependencies to exclude from
//! the default-features check. This is useful for dependencies that you explicitly want to have
//! default features enabled.

mod validation;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use validation::validate_workspace_dependencies;

/// Cargo subcommand to ensure workspace dependencies have default-features = false
#[derive(Parser)]
#[command(name = "cargo-ensure-no-default-features")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ensure all workspace dependencies have default-features = false
    EnsureNoDefaultFeatures {
        /// Path to Cargo.toml
        #[arg(long, default_value = "Cargo.toml", value_name = "PATH")]
        manifest_path: PathBuf,

        /// List of dependencies to exclude from default-features check
        #[arg(long, short = 'e', value_delimiter = ',')]
        exceptions: Option<Vec<String>>,
    },
}

// tested by integration tests
#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::EnsureNoDefaultFeatures { manifest_path, exceptions } => {
            let content = std::fs::read_to_string(&manifest_path).with_context(|| format!("Failed to read {}", manifest_path.display()))?;
            let exceptions = exceptions.unwrap_or_default();

            let (errors, found_deps) = validate_workspace_dependencies(&content, &exceptions)?;
            if !errors.is_empty() {
                eprintln!("❌ Found {} dependencies without default-features = false:\n", errors.len());
                for error in &errors {
                    eprintln!("{error}");
                }
                std::process::exit(1);
            }

            // Warn if any exception was not found in the dependencies
            for exception in &exceptions {
                if !found_deps.contains(exception) {
                    eprintln!("⚠️ Warning: exception '{exception}' was not found in [workspace.dependencies]");
                }
            }

            println!("✅ All required workspace dependencies have default-features = false");
        }
    }

    Ok(())
}
