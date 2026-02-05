# CLAUDE.md - AI Assistant Context for Essence Wars

<!-- Last verified: 2026-02-05 -->

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine. Written in Rust with focus on performance and correctness.

**Author:** Christian Wissmann (Chris), Best Friends with Claude

**Monorepo structure** - Each subproject has its own CLAUDE.md with focused context:
- `crates/cardgame/CLAUDE.md` - Core engine, bots, tuning, balance tools
- `crates/essence-wars-ui/CLAUDE.md` - Tauri desktop app, Svelte 5 UI
- `crates/essence-wars-mcp/CLAUDE.md` - MCP server for Claude Code integration
- `python/CLAUDE.md` - Python bindings, ML agents, training
- `docs/essence-wars-design.md` - Game rules, factions, cards (reference)

## Quick Commands

```bash
# Build & Test
cargo build --release                    # Full workspace
cargo nextest run --status-level=fail    # Rust tests
uv run pytest python/tests               # Python tests

# Lint (all configured for minimal output)
cargo lint                               # Rust (clippy alias)
uv run mypy python/essence_wars          # Python types
uv run ruff check python/essence_wars    # Python lint
pnpm run check                           # Svelte/TS (in crates/essence-wars-ui)
pnpm run lint                            # ESLint (in crates/essence-wars-ui)

# Key binaries (run with --help for options)
cargo run --release --bin arena --        # Bot matches
cargo run --release --bin validate --     # Quick balance check
cargo run --release --bin benchmark --    # Thorough balance analysis
cargo run --release --bin swiss --        # Tournament mode
cargo run --release --bin replay --       # Game replay/debug
```

## AI-Friendly Tooling

This project is configured for token-efficient CLI output. Verbose output (colors, progress bars) wastes context tokens.

| Tool | Config | Effect |
|------|--------|--------|
| cargo | `.cargo/config.toml` | No colors, no progress bars |
| pytest | `pyproject.toml` | Quiet mode, short tracebacks |
| eslint | `package.json` | `--quiet` (errors only) |
| svelte-check | `package.json` | `--output human` (less verbose) |

Environment variables in `.envrc` (for direnv): `NO_COLOR=1`, `CARGO_TERM_COLOR=never`, etc.

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `crates/cardgame/` | Core game engine, bots, CLI binaries |
| `crates/essence-wars-mcp/` | MCP server for Claude Code |
| `crates/essence-wars-ui/` | Tauri 2 + Svelte 5 desktop app |
| `python/essence_wars/` | Python bindings, ML agents |
| `data/cards/` | Card definitions (YAML) |
| `data/decks/` | Deck definitions (TOML) |
| `data/weights/` | Tuned bot weights |
| `docs/` | Design docs, guides |

## Test Organization

**IMPORTANT**: Rust unit tests are **separate from source code** (not inline `#[cfg(test)]`).

- **Unit tests**: `crates/cardgame/tests/unit/<module>_tests.rs`
- **Integration tests**: `crates/cardgame/tests/*.rs`
- **Shared utilities**: `crates/cardgame/tests/common/mod.rs`

When adding tests, create in `tests/unit/` and add module to `tests/unit.rs`.

## Logging

```bash
RUST_LOG=info cargo run --release --bin arena -- ...   # Weight loading info
RUST_LOG=debug cargo run --release --bin tune -- ...   # Verbose debugging
```

## Versioning

Version in **root `Cargo.toml`** `[workspace.package]`.

| Change | Bump |
|--------|------|
| Game rules/API | MINOR |
| New features | MINOR |
| Bug fixes | PATCH |

**Update checklist**: Root `Cargo.toml` → `crates/cardgame/src/version.rs` test → Run tests
