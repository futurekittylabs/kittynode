# Kittynode Developer Reference

This file contains all development guidelines and instructions for working with the Kittynode codebase. It serves both human developers and AI assistants.

## Development Setup
```bash
# Prerequisites: Rust, just, Node.js, bun
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
- Logging: Primarily use the custom toast util so the user can see the necessary info (and report bugs); can also use `console.info()` and `console.error()` when appropriate
- Always log errors when calling library functions or external APIs

### Architectural Principles
- Core library provides functionality to all frontends
- Direct container access through Bollard instead of Docker CLI
- Modular package ecosystem that's secure, consistent, and atomic
- Testing focuses on unit tests and behavior tests, not 100% coverage

## Testing Philosophy

### Test Types
- **Unit tests** for kittynode-core: Focus on data transformation without mocking
- **Behavior tests** for kittynode-cli: Test actual CLI behavior

We discourage mocking in unit tests. A unit test should take primitive data, transform it, and return a result. Integration boundaries are tested at higher levels.

### Code Coverage
We include integration test coverage in our total. This represents our holistic testing strategy. We don't aim for 100% coverage - tests should explain how code works and ensure expected behavior.

### Running Tests
```bash
just test                   # Run unit tests
just test-all               # Run all tests including ignored
cargo nextest run test_name # Run a specific test
```

## Logging Guidelines

### When to Log
- **Required**: In library functions and after operations succeed/fail
- **Encouraged**: At API entry points in binding layers
- **Optional**: Other layers (use judgment, avoid over-logging)

### Log Format
- Write logs in sentence case without trailing periods
- Log errors when calling library functions or external APIs

### Log Functions
- **Rust**: `info!()` and `error!()`
- **JavaScript**: `console.info()` and custom `error()` functions

## Managing Releases

We push tags in format `<package-name>-0.y.z-alpha` to GitHub. CI builds and publishes draft releases with auto-generated changelogs.

Find latest releases at: https://github.com/blackkittylabs/kittynode/releases

## Troubleshooting

### iOS Deployment
1. Ensure physical device is connected and Xcode is open
2. Uninstall any previous builds from device
3. If iOS IP isn't visible, check connection

### iOS Clean Commands
```bash
# Clean Xcode data
rm -rf ~/Library/Developer/Xcode/DerivedData
rm -rf ~/Library/Developer/Xcode/Archives
rm -rf ~/Library/Developer/Xcode/Projects

# Reset simulators
xcrun simctl erase all
```

## Git Style Guidelines
- We use merge commits for PRs
- We don't use conventional commits
