# Build & Run Scripts

Collection of scripts for building, testing, and running Essence Wars components.

## UI Development & Building

### Development Mode (Hot Reload)
```bash
./scripts/launch-ui.sh    # Start Tauri dev server with hot reload
```

### Release Builds

**Linux:**
```bash
./scripts/build-linux.sh              # Build AppImage + .deb
./scripts/build-linux.sh --appimage   # Build AppImage only
./scripts/build-linux.sh --deb        # Build .deb package only
```
Output: `target/release/bundle/appimage/essence-wars-ui_*.AppImage` (200 MB)

**Windows (from Linux/WSL):**
```bash
./scripts/build-windows.sh            # Build unsigned .exe
./scripts/build-windows.sh --sign     # Build and sign .exe
```
Output: `target/x86_64-pc-windows-gnu/release/essence-wars-ui.exe`

**Run Release Build:**
```bash
./scripts/run-ui.sh         # Run latest build (detects platform)
./scripts/run-ui.sh --build # Build first, then run
```

## Engine & Tests

**Run Tests:**
```bash
./scripts/run-tests.sh          # Full test suite
cargo nextest run               # Faster test runner (preferred)
cargo test                      # Standard cargo test
```

**Linting:**
```bash
./scripts/run-clippy.sh         # Lint production code (excludes tests)
```

**Benchmarks:**
```bash
./scripts/run-benchmarks.sh     # Full benchmark suite with report
cargo bench -p cardgame         # Criterion benchmarks only
```

## Tuning & Arena

**Tune Bot Weights:**
```bash
# Generalist (recommended)
cargo run --release --bin tune -- --mode generalist --tag baseline --generations 100

# Specialist for specific matchup
cargo run --release --bin tune -- --mode specialist \
  --deck broodmother_pack --opponent architect_fortify --tag aggro

# Faction specialist
cargo run --release --bin tune -- --mode faction-specialist \
  --faction argentum --tag argentum_v1
```

**Run Arena Matches:**
```bash
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100 --progress
```

## Art Generation

**Generate Card Art:**
```bash
./scripts/generate-card-art.py
```

**Generate Commander Art:**
```bash
./scripts/generate-commander-art.py
```

## Python Environment

**Setup:**
```bash
uv sync --all-groups           # Install all dependencies
```

**Run Python Tools:**
```bash
uv run pytest python/tests -v  # Run Python tests
uv run ruff check python/      # Lint Python code
```

## Prerequisites

### Linux Development
```bash
# Tauri dependencies
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Build tools
sudo apt install build-essential pkg-config pnpm
```

### Windows Cross-Compilation (from Linux/WSL)
```bash
# Install Rust target
rustup target add x86_64-pc-windows-gnu

# Install MinGW
sudo apt install mingw-w64 nsis osslsigncode
```

## Project Structure
```
scripts/
├── build-linux.sh           # Build Linux AppImage/deb
├── build-windows.sh         # Build Windows .exe (from Linux/WSL)
├── run-ui.sh                # Run release builds
├── launch-ui.sh             # Dev mode with hot reload
├── run-tests.sh             # Test suite runner
├── run-clippy.sh            # Linter
├── run-benchmarks.sh        # Benchmark suite
├── generate-card-art.py     # Card art generation
└── generate-commander-art.py # Commander art generation
```
