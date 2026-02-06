# Contributing to Essence Wars

Thanks for your interest in contributing to Essence Wars!

## Repository Size Note

The full repository clone is approximately **170 MB**, primarily due to game assets (card art, music, sound effects). This is intentional - we bundle assets for easy development setup.

| Component | Size | Required For |
|-----------|------|--------------|
| Source code | ~8 MB | All development |
| Card art | ~61 MB | UI development |
| Music/audio | ~44 MB | UI development |
| Other assets | ~9 MB | UI development |

**Engine-only or Python-only contributors**: If you're only working on the Rust engine or Python bindings and don't need the UI assets, you can use a sparse checkout:

```bash
# Clone without checking out files
git clone --no-checkout https://github.com/christianWissmann85/essence-wars.git
cd essence-wars

# Configure sparse checkout
git sparse-checkout init --cone
git sparse-checkout set crates/cardgame crates/essence-wars-mcp python data docs

# Checkout
git checkout master
```

This reduces the checkout to ~15 MB.

## Development Areas

### Rust Game Engine (`crates/cardgame/`)

The core deterministic game engine, bots, and CLI tools.

```bash
# Build
cargo build --release

# Run tests
cargo nextest run --status-level=fail

# Lint
cargo lint  # alias for clippy

# Run arena matches
cargo run --release --bin arena -- --help
```

See `crates/cardgame/CLAUDE.md` for detailed engine documentation.

### Python Bindings & ML (`python/`)

Python bindings for the game engine, ML agents, and training scripts.

```bash
# Setup (using uv)
uv venv && source .venv/bin/activate
uv pip install -e ".[dev]"

# Run tests
uv run pytest python/tests

# Type checking
uv run mypy python/essence_wars

# Lint
uv run ruff check python/essence_wars
```

See `python/CLAUDE.md` for Python development details.

### Desktop UI (`crates/essence-wars-ui/`)

Tauri 2 + Svelte 5 desktop application.

```bash
cd crates/essence-wars-ui

# Install dependencies
pnpm install

# Development with hot reload
pnpm tauri:dev

# Type checking
pnpm run check

# Lint
pnpm run lint

# Build for current platform
pnpm tauri:build
```

See `crates/essence-wars-ui/CLAUDE.md` for UI development details.

### MCP Server (`crates/essence-wars-mcp/`)

Claude Code integration via Model Context Protocol.

```bash
cargo build --release --bin essence-wars-mcp
```

See `crates/essence-wars-mcp/CLAUDE.md` for MCP documentation.

## Code Style

- **Rust**: Follow `cargo clippy` suggestions, use `cargo fmt`
- **Python**: Use `ruff` for linting, `mypy` for type checking
- **TypeScript/Svelte**: Use `eslint` and `svelte-check`

## Testing

All PRs should pass the relevant test suites:

```bash
# Rust
cargo nextest run --status-level=fail

# Python
uv run pytest python/tests

# UI (type checking)
cd crates/essence-wars-ui && pnpm run check
```

## Commits

- Write clear, concise commit messages
- Reference issues where applicable (`Fixes #123`)

## Questions?

Open an issue or discussion on GitHub. We're happy to help!
