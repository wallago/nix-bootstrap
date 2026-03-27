# List all commands
default:
    @just --list

# ── Dev workflow ──────────────────────────────────────────────

# Background checker (shows errors live as you save)
watch:
    bacon

# Run the project
run *ARGS:
    cargo run -- {{ARGS}}

# ── Quality ───────────────────────────────────────────────────

# Run all tests with nextest
test:
    cargo nextest run

# Lint with clippy (mirrors CI)
lint:
    cargo clippy --all-targets -- -D warnings
    taplo check
    cargo machete

# Format everything
fmt:
    cargo fmt
    taplo fmt

# Run tests with coverage report
coverage:
    cargo tarpaulin --out html --output-dir target/coverage
    @echo "📊 Report: target/coverage/tarpaulin-report.html"

# ── Security & compliance ─────────────────────────────────────

# License + advisory + source checks
deny:
    cargo deny check

# Security audit
audit:
    cargo audit

# ── Full pipeline ─────────────────────────────────────────────

# Run the full CI pipeline locally (mirrors nix flake check)
ci: fmt lint test deny audit
    @echo "✅ All checks passed"

# Run the full pipeline via Nix (exactly what CI runs)
ci-nix:
    nix flake check --print-build-logs

# ── Build ─────────────────────────────────────────────────────

# Build release binary via Nix
build:
    nix build

# Build release binary via cargo
build-cargo:
    cargo build --release

# ── Analysis ──────────────────────────────────────────────────

# Profile with flamegraph
flamegraph:
    cargo flamegraph

# Analyze binary size (top 20 functions)
bloat:
    cargo bloat --release -n 20

# Expand macros for a given item
expand ITEM:
    cargo expand {{ITEM}}

# Code stats
stats:
    tokei

# Check for typos in code + docs
typos:
    typos

# Check links in markdown files
links:
    lychee *.md

# ── Release ───────────────────────────────────────────────────

# Generate changelog (dry run)
changelog:
    git cliff --unreleased

# Generate changelog and write to CHANGELOG.md
changelog-write:
    git cliff -o CHANGELOG.md

# Lint the last commit message
commit-check:
    committed HEAD
