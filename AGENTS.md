# Kittynode Developer Reference

## Git instructions

- NEVER use conventional commits like `feat`, `fix`, etc. Use descriptive commit messages instead.

## Architecture

- If needed, read the architecture documentation here: `./docs/src/content/docs/reference/architecture.mdx`.
- If making changes to the architecture, update the architecture documentation.

### Remote access

- The desktop toggle now starts/stops `kittynode-web` via the CLI binary. When enabled, the service listens on the port reported in Settings/System Info (default 3000).
- `~/.kittynode/runtime/kittynode-web.json` tracks PID/port, and logs stream to `~/.kittynode/runtime/kittynode-web.log`.
- Expose the port only on trusted networks; the UI surfaces errors from the toggle so operators can react quickly.
- The toggle requires the CLI binary (`kittynode`/`kittynode-cli`) to be on disk or provided via `KITTYNODE_CLI_PATH`.
- A per-launch service token is injected via the `KITTYNODE_WEB_SERVICE_TOKEN` environment variable; ensure untrusted users cannot read parent process memory/environment.

## Development Setup

```bash
# Prerequisites: Rust, just, Node.js, bun
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install just

# Clone and set up
git clone git@github.com:futurekittylabs/kittynode.git && cd kittynode
just setup
```

## Command Reference

- For the most up-to-date list of commands, run `just -l` or check the justfile directly.

## Code Style Guidelines

- Always run the lint tool after making changes, we have `just lint-js` and `just lint-rs`.

### Rust

- Use `cargo fmt` standards with clippy for linting
- Prefer `Result<T, Error>` for error handling
- Group imports: std, external crates, project modules
- Logging: Use `info!()` and `error!()` in sentence case without trailing periods
- Testing: Focus on data transformation, avoid mocking in unit tests
- Use eyre properly for errors, only use unwrap/expect when absolutely necessary

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
- **Optional**: Other layers (use judgment, avoid over-logging, but log sufficiently)

### Log Format

- Write logs in sentence case without trailing periods
- Log errors when calling library functions or external APIs

### Log Functions

- **Rust**: `info!()` and `error!()`
- **JavaScript**: `console.info()` and custom `error()` functions

## Managing Releases

- We push tags in the format `<package-name>@0.y.z` to GitHub. CI builds and publishes draft releases with auto-generated changelogs.
- We use `just release` to create a new release and push it up.

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
