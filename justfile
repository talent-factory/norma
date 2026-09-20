# norma — Task Runner
#
# Prereqs:
#   just  (Homebrew: `brew install just`)
#   Rust toolchain pinned in rust-toolchain.toml (rustup installs it automatically)
#   cargo-watch, for `just watch` (Homebrew/cargo: `cargo install cargo-watch`)
#   pre-commit, for `just install-hooks` (https://pre-commit.com/)
#
# List all recipes grouped by category:  `just`

set positional-arguments

# Default recipe shows the grouped recipe list.
default:
    @just --list --unsorted

# ──────────────────────── Build ────────────────────────

[group('Build')]
build:
    cargo build

[group('Build')]
build-release:
    cargo build --release

# Installs the `norma` binary onto $PATH via `cargo install --path .`.
[group('Build')]
install:
    cargo install --path .

# ──────────────────────── Test ────────────────────────

[group('Test')]
test:
    cargo test

[group('Test')]
test-lib:
    cargo test --lib

# norma validates its own src/ with its own Rust "no debug print" pattern.
[group('Test')]
test-dogfooding:
    cargo test --test dogfooding -- --nocapture

# Behavioral coverage for the 16 GoF default patterns (Singleton/Factory/Observer/Strategy).
[group('Test')]
test-gof-patterns:
    cargo test --test gof_patterns -- --nocapture

# Rebuilds and re-runs the test suite on every source change.
[group('Test')]
watch:
    cargo watch -x build -x test

# ──────────────────────── Lint & Format ────────────────────────

[group('Lint & Format')]
fmt:
    cargo fmt

[group('Lint & Format')]
fmt-check:
    cargo fmt --all -- --check

[group('Lint & Format')]
lint:
    cargo clippy --all-targets

# ──────────────────────── Quality ────────────────────────

# Full local check chain: format check + lint + tests. Run before committing.
[group('Quality')]
check: fmt-check lint test
    @echo "✓ All checks passed."

# ──────────────────────── Dev ────────────────────────

# Runs the MCP tool server on stdio with debug tracing (logs go to stderr).
[group('Dev')]
serve:
    RUST_LOG=debug cargo run -- serve

[group('Dev')]
list-patterns:
    cargo run -- list-patterns

# Example: just validate src/main.rs rust
[group('Dev')]
validate file language="rust":
    cargo run -- validate --language {{language}} {{file}}

# Builds and opens norma's own rustdoc in the browser.
[group('Dev')]
doc:
    cargo doc --open

# ──────────────────────── Setup ────────────────────────

[group('Setup')]
install-hooks:
    pre-commit install

[group('Setup')]
setup: install install-hooks
    @echo "✓ Setup complete. Try: just list-patterns"

# ──────────────────────── Clean ────────────────────────

[group('Clean')]
clean:
    cargo clean
