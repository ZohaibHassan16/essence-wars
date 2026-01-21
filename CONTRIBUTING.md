# Contributing to Essence Wars

Thank you for your interest in contributing to Essence Wars! This document provides guidelines for contributing to the project.

## Getting Started

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/christianWissmann85/essence-wars
   cd essence-wars
   ```

2. **Install Rust toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```

3. **Build the project**
   ```bash
   cargo build --release
   ```

4. **Set up Python environment**
   ```bash
   # Install uv (recommended)
   curl -LsSf https://astral.sh/uv/install.sh | sh

   # Create virtual environment and install dependencies
   uv venv
   source .venv/bin/activate
   uv pip install -e ".[train,analysis]"
   ```

5. **Run tests to verify setup**
   ```bash
   cargo nextest run --status-level=fail
   ```

## Code Organization

```
essence-wars/
├── crates/
│   ├── cardgame/           # Core game engine (Rust)
│   │   ├── src/
│   │   │   ├── core/       # Game types, state, actions
│   │   │   ├── engine/     # Game loop, effects
│   │   │   ├── bots/       # AI players
│   │   │   └── bin/        # CLI tools
│   │   └── tests/unit/     # Unit tests (separate from src!)
│   └── essence-wars-3d/    # Bevy 3D client
├── python/
│   └── essence_wars/       # Python bindings and ML
│       ├── agents/         # PPO, AlphaZero, Neural MCTS
│       └── scripts/        # Training scripts
├── data/
│   ├── cards/              # Card definitions (YAML)
│   ├── decks/              # Deck definitions (TOML)
│   └── datasets/           # Training datasets
├── docs/                   # Documentation
└── models/                 # Trained model checkpoints
```

## Development Workflow

### Before Making Changes

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Check existing issues** to avoid duplicate work

### Making Changes

1. **Write tests first** when adding new features
2. **Follow existing code style** - run linters before committing
3. **Keep commits focused** - one logical change per commit

### Testing

```bash
# Run all Rust tests
cargo nextest run --status-level=fail

# Run with verbose output
cargo nextest run

# Run specific test
cargo nextest run test_name

# Run Python tests
pytest python/tests/

# Run stress tests (longer)
./scripts/run-tests.sh medium
```

### Linting

```bash
# Rust
./scripts/run-clippy.sh
cargo fmt --check

# Python
ruff check python/
ruff format --check python/
```

### Submitting Changes

1. **Run all tests** and ensure they pass
2. **Update documentation** if needed
3. **Create a pull request** with a clear description
4. **Reference any related issues**

## Code Style

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Document public APIs with doc comments

### Python

- Follow PEP 8
- Use type hints for function signatures
- Use `ruff` for linting and formatting
- Document functions with docstrings

### Tests

**Important**: Unit tests are **separate from source code** in `tests/unit/`:
```
crates/cardgame/
├── src/
│   └── core/game_state.rs      # Source code
└── tests/
    └── unit/
        └── game_state_tests.rs  # Tests for game_state.rs
```

When adding tests:
1. Create file in `tests/unit/`
2. Add module to `tests/unit.rs`

## Types of Contributions

### Bug Fixes

1. Create an issue describing the bug
2. Include reproduction steps
3. Submit a PR referencing the issue

### New Features

1. **Discuss first** - open an issue to discuss the feature
2. Consider backward compatibility
3. Add tests and documentation

### Documentation

- Fix typos, clarify explanations
- Add examples and tutorials
- Update outdated information

### AI/ML Contributions

- **New training algorithms**: Add to `python/essence_wars/agents/`
- **New bot types**: Add to `crates/cardgame/src/bots/`
- **Trained models**: Submit via the [leaderboard](docs/SUBMIT_AGENT.md)
- **Datasets**: Share on HuggingFace with proper documentation

## Submitting Trained Agents

To submit a trained agent to the leaderboard:

```bash
python -m essence_wars.scripts.submit_agent \
    --checkpoint path/to/model.pt \
    --name "Your Agent Name" \
    --repo-id yourusername/agent-repo
```

See [SUBMIT_AGENT.md](docs/SUBMIT_AGENT.md) for details.

## Versioning

The project uses semantic versioning:
- **MAJOR**: Breaking API changes
- **MINOR**: New features, game rule changes
- **PATCH**: Bug fixes

Version is defined in root `Cargo.toml` and shared across all crates.

## Communication

- **Issues**: Bug reports, feature requests
- **Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, feel free to open an issue or start a discussion. We're happy to help!
