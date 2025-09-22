# build all crates
build:
  cargo build

# build the desktop app
build-app:
  cargo tauri build

# start the docs dev server
docs:
  bun --cwd docs dev --open

# generate the kittynode-core docs
docs-rs:
  cargo doc -p kittynode-core

# start the desktop app
app:
  cargo tauri dev

# install icons
icons:
  cargo tauri icon assets/kittynode-logo-app.png
  cargo tauri icon assets/kittynode-logo-square.png --ios-color '#A181A7' -o tmp
  mv tmp/ios/* packages/app/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset
  rm -rf tmp

# install or update dev tools
install-dev-tools:
  cargo install cargo-edit cargo-llvm-cov cargo-nextest just tauri-cli

# start the ios app on a physical device
ios:
  cargo tauri ios dev --force-ip-prompt -vvv

# make an ios build
ios-build:
  cargo tauri ios build

# init the ios app
ios-init:
  cargo tauri ios init
  cp -R packages/app/src-tauri/gen-overrides/gen/* packages/app/src-tauri/gen
  just icons

# start the ios app on a virtual device
ios-virtual:
  cargo tauri ios dev 'iPhone 16'

# run the kittynode cli with the given args
kittynode *args='':
  @if [ -z "{{args}}" ]; then target/debug/kittynode help; else target/debug/kittynode {{args}}; fi

# lint the javascript code
lint-js:
  bun -F docs -F app -F website format-lint && bun -F app -F website check

# lint and fix the javascript code
lint-js-fix:
  bun -F docs -F app -F website format-lint:fix && bun -F app -F website check

# lint the rust code
lint-rs:
  cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check

# lint the rust code with pedantic rules
lint-rs-pedantic:
  cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::too_many_lines && cargo fmt --all -- --check

# release crates + desktop app
release:
  #!/usr/bin/env bash
  set -euxo pipefail
  cargo set-version --bump minor
  ver="$(cargo pkgid -p kittynode-tauri | cut -d@ -f2)"
  git add $(git ls-files "*/Cargo.toml") Cargo.toml Cargo.lock
  git commit -m "Release crates + kittynode-app@${ver}"
  git tag "kittynode-app@${ver}" -m "Release crates + kittynode-app@${ver}"
  cargo publish -p kittynode-core --dry-run && cargo publish -p kittynode-core
  cargo publish -p kittynode-cli --dry-run && cargo publish -p kittynode-cli --locked
  cargo publish -p kittynode-tauri --dry-run && cargo publish -p kittynode-tauri --locked
  cargo publish -p kittynode-web --dry-run && cargo publish -p kittynode-web --locked
  git push origin HEAD "kittynode-app@${ver}"

# set up the project
setup:
  bun install && just install-dev-tools && just ios-init

# add a shadcn component
shadcn-add *args='':
  cd packages/app && bunx shadcn-svelte@latest add {{args}} && bun format-lint:fix

# update shadcn components
shadcn-update:
  cd packages/app && bunx shadcn-svelte@latest update -a -y && bun format-lint:fix

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

# update all toolchains and dependencies
update:
  rustup update
  just install-dev-tools
  cargo upgrade
  bun upgrade
  bun update

# start the web server
web:
  cargo run -p kittynode-web

# start the website
website:
  bun --cwd website dev --open
