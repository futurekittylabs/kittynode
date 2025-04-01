# Kittynode Development Guide

## Project Overview
Kittynode is a control center for world computer operators with these main components:
- `kittynode-core`: Core Rust library powering all applications
- `kittynode-cli`: Command-line interface built on the core library
- `kittynode-gui`: Cross-platform Tauri app with Svelte frontend
- `kittynode-web`: Web server binding to the core library

## Development Setup
```bash
# Prerequisites: Rust, just, Node.js, pnpm
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install just

# Clone and set up
git clone git@github.com:blackkittylabs/kittynode.git && cd kittynode
just setup
```

## Command Reference
For the most up-to-date list of commands, run `just -l` or check the justfile directly.
Common commands include:

```bash
# Build & Run
just build                  # Build all crates  
just gui                    # Start desktop app (dev)
just docs                   # Start docs server

# Testing
just test                   # Run unit tests
just test-all               # Run all tests including ignored
cargo nextest run test_name # Run a specific test

# Linting
just lint-rs                # Lint Rust code
just lint-js                # Lint JS/TS code
```

## Code Style Guidelines

### Rust
- Use `cargo fmt` standards with clippy for linting
- Prefer `Result<T, Error>` for error handling
- Group imports: std, external crates, project modules
- Logging: Use `info!()` and `error!()` in sentence case without trailing periods
- Testing: Focus on data transformation, avoid mocking in unit tests

### TypeScript/JavaScript
- Format with Biome using double quotes and 2-space indentation
- Use camelCase for variables/functions
- Logging: Use `console.info()` and custom `error()` functions
- Always log errors when calling library functions or external APIs

### Architectural Principles
- Core library provides functionality to all frontends
- Direct container access through Bollard instead of Docker CLI
- Modular package ecosystem that's secure, consistent, and atomic
- Testing focuses on unit tests and behavior tests, not 100% coverage

## Documentation
This file consolidates information previously located in:
- `./justfile` - Command reference
- `./docs/src/content/docs/development/development-guide.mdx` - Development guide
- `./packages/README.md` - Architecture overview

For end-user documentation, visit https://kittynode.io

## Git Style Guidelines
- We use merge commits for PRs
- We don't use conventional commits
