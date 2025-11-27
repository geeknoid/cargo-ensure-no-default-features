#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! Eliminate superfluous features in a Rust workspace.
//!
//! This tool checks that all workspace dependencies in Cargo.toml have
//! `default-features = false`. This is a best practice in repos that publish multiple independent
//! crates, to ensure that each individual crate has the minimal set of features they need.
//! This helps improve build times for any consumers of these crates by avoiding
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
        #[arg(long, default_value = "Cargo.toml")]
        manifest_path: PathBuf,
    },
}

// tested by integration tests
#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::EnsureNoDefaultFeatures { manifest_path } => {
            let cargo_toml_path = manifest_path;
            let content =
                std::fs::read_to_string(&cargo_toml_path).with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

            let errors = validate_workspace_dependencies(&content)?;

            if !errors.is_empty() {
                eprintln!("❌ Found dependencies without default-features = false:\n");
                for error in &errors {
                    eprintln!("{error}");
                }
                eprintln!("\nAll workspace dependencies must have default-features = false.");
                eprintln!("Individual crates can enable features they need in their own Cargo.toml.");
                eprintln!("\nFound {} dependency validation error(s)", errors.len());
                std::process::exit(1);
            }
            println!("✅ All workspace dependencies have default-features = false");
        }
    }
    Ok(())
}
