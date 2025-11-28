# cargo-ensure-no-default-features

[![crate.io](https://img.shields.io/crates/v/cargo-ensure-no-default-features.svg)](https://crates.io/crates/cargo-ensure-no-default-features)
[![CI](https://github.com/geeknoid/cargo-ensure-no-default-features/workflows/main/badge.svg)](https://github.com/geeknoid/cargo-ensure-no-default-features/actions)
[![Coverage](https://codecov.io/gh/geeknoid/cargo-ensure-no-default-features/graph/badge.svg?token=FCUG0EL5TI)](https://codecov.io/gh/geeknoid/cargo-ensure-no-default-features)
[![Minimum Supported Rust Version 1.88](https://img.shields.io/badge/MSRV-1.88-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

<!-- cargo-rdme start -->

Eliminate superfluous features in a Rust workspace.

This tool checks that all workspace dependencies in Cargo.toml have
`default-features = false`. This is a best practice in repos that publish multiple independent
crates, to ensure that each individual crate has the minimal set of features they need.
This can improve build times for any consumers of these crates by avoiding
unnecessary features being enabled by default.

Install with:

```bash
cargo install cargo-ensure-no-default-features
```

And use with:

```bash
cargo ensure-no-default-features
```

The --manifest-path option lets you specify an explicit Cargo.toml file to check. Without this
option, it defaults to the Cargo.toml in the current directory.

The --exceptions option lets you specify a comma-separated list of dependencies to exclude from
the default-features check. This is useful for dependencies that you explicitly want to have
default features enabled.

<!-- cargo-rdme end -->
