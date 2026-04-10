# build all crates
build:
    cargo build

# start the docs dev server
kittynode-docs:
    bun --cwd kittynode-docs dev --open

# generate the kittynode-core docs
docs-rs:
    cargo doc -p kittynode-core

# install dev tools
install-dev-tools:
    cargo install cargo-edit cargo-llvm-cov cargo-nextest just

# run the kittynode cli with the given args
kittynode *args='':
    @if [ -z "{{ args }}" ]; then target/debug/kittynode help; else target/debug/kittynode {{ args }}; fi

# lint the javascript code
lint-js:
    bun -F kittynode-docs -F kittynode-com format-lint && bun -F kittynode-com check

# lint and fix the javascript code
lint-js-fix:
    bun -F kittynode-docs -F kittynode-com format-lint:fix && bun -F kittynode-com check

# lint the rust code
lint-rs:
    cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check

# lint the rust code with pedantic rules
lint-rs-pedantic:
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::too_many_lines && cargo fmt --all -- --check

# release cli
release-cli:
    #!/usr/bin/env bash
    set -euxo pipefail
    git pull --rebase origin main
    cargo set-version --bump minor -p kittynode-cli
    git add $(git ls-files "*/Cargo.toml") Cargo.toml Cargo.lock
    verCli="$(cargo pkgid -p kittynode-cli | cut -d@ -f2)"
    git commit -m "Release kittynode-cli-${verCli}"
    git tag "kittynode-cli-${verCli}" -m "Release kittynode-cli-${verCli}"
    git push origin HEAD "kittynode-cli-${verCli}"

# set up the project
setup:
    bun install && just install-dev-tools

# run the unit tests
test:
    cargo nextest run

# run the unit and integration tests
test-all:
    cargo nextest run -- --include-ignored

# run the unit tests with coverage
test-coverage:
    cargo llvm-cov nextest

# run the unit and integration tests with coverage
test-coverage-all:
    cargo llvm-cov nextest -- --include-ignored

# show coverage with codecov percentage at the end
coverage:
    cargo llvm-cov --all-features --workspace 2>&1 | awk '{print} /^TOTAL/ {pct=$4} END {print "\ncodecov: " pct}'

# update dependencies
update:
    nix flake update
    cargo upgrade
    cd kittynode-com && bun update --latest
    cd kittynode-docs && bun update --latest

# start the web server
web:
    cargo run -p kittynode-web

# start the dot-com site
kittynode-com:
    bun --cwd kittynode-com dev --open
