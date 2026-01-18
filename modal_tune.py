#!/usr/bin/env python3
"""
Modal Cloud Training for Essence Wars
Runs training pipeline with optional validation phase.

Usage:
    modal run modal_tune.py                         # Full pipeline (train + validate)
    modal run modal_tune.py --mode train-only       # Train only, skip validation
    modal run modal_tune.py --mode validate-only    # Validate with existing weights
    modal run modal_tune.py --single generalist     # Run single training config
    modal deploy modal_tune.py                      # Deploy as persistent app
"""

import sys
from pathlib import Path

import modal

# ============================================================================
# Configuration
# ============================================================================

APP_NAME = "essence-wars-tuning"
RUST_VERSION = "stable"  # Use latest stable Rust

# CPU configuration (scale up for faster training)
CPU_COUNT = 16  # 16 cores per training job (excellent parallel performance)
MEMORY_MB = 8192  # 8 GB RAM (sufficient for MCTS)
TIMEOUT_SECONDS = 7200  # 2 hours max (usually finishes in 10-15 min)

# Validation configuration (defaults, can be overridden via CLI)
VALIDATION_CPU = 32  # 32 cores for faster validation (adjustable via --cores)
VALIDATION_MEMORY = 16384  # 16 GB
VALIDATION_TIMEOUT = 3600  # 1 hour (adjustable via --validation-timeout)
VALIDATION_GAMES = 500  # Games per matchup per direction (adjustable via --validation-games)
                        # Round-robin: 40 deck matchups × 2 directions = 40k total games at default
VALIDATION_MCTS_SIMS = 100

# Training configurations
TRAINING_CONFIGS = [
    {
        "tag": "generalist-v0.4-modal",
        "mode": "generalist",
        "args": ["--generations", "100", "--games", "150", "--mcts-sims", "25"],
        "description": "Universal weights for all decks",
    },
    {
        "tag": "argentum-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "argentum", "--generations", "100", "--games", "150", "--mcts-sims", "25"],
        "description": "Argentum (defensive control) specialist",
    },
    {
        "tag": "symbiote-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "symbiote", "--generations", "100", "--games", "150", "--mcts-sims", "25"],
        "description": "Symbiote (aggressive tempo) specialist",
    },
    {
        "tag": "obsidion-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "obsidion", "--generations", "100", "--games", "150", "--mcts-sims", "25"],
        "description": "Obsidion (burst damage) specialist",
    },
]

# ============================================================================
# Modal App Setup
# ============================================================================

app = modal.App(APP_NAME)

# Build custom image with Rust toolchain
rust_image = (
    modal.Image.debian_slim(python_version="3.11")
    .apt_install(
        "curl",
        "build-essential",
        "git",
        "pkg-config",
        "libssl-dev",
    )
    .run_commands(
        # Install Rust
        f"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {RUST_VERSION}",
        # Add cargo to PATH
        "echo 'source $HOME/.cargo/env' >> ~/.bashrc",
    )
    .env({"PATH": "/root/.cargo/bin:$PATH"})
)

# Create shared volume for experiment outputs (persists across runs)
volume = modal.Volume.from_name("essence-wars-experiments", create_if_missing=True)

# ============================================================================
# Helper Functions (used by Modal functions)
# ============================================================================

def extract_workspace(workspace_snapshot: bytes, path: str = "/tmp/essence-wars") -> Path:
    """
    Extract workspace snapshot tarball to specified path.
    
    Args:
        workspace_snapshot: Gzipped tarball bytes
        path: Destination path for extraction
        
    Returns:
        Path object pointing to extracted workspace
    """
    import io
    import tarfile

    print("\n📦 Extracting workspace...")
    workspace_path = Path(path)
    workspace_path.mkdir(exist_ok=True)

    with tarfile.open(fileobj=io.BytesIO(workspace_snapshot), mode='r:gz') as tar:
        tar.extractall(workspace_path)

    print(f"✓ Workspace extracted to {workspace_path}")
    return workspace_path


def build_rust_binary(workspace: Path, binary: str) -> tuple[bool, str, float]:
    """
    Build Rust binary in release mode using cargo.
    
    Args:
        workspace: Path to workspace with Cargo.toml
        binary: Name of binary to build (e.g., 'tune', 'validate')
        
    Returns:
        Tuple of (success: bool, stderr: str, build_time: float)
    """
    import subprocess
    import time

    print(f"\n🔨 Building {binary} binary...")
    build_start = time.time()

    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", binary],
        cwd=workspace,
        capture_output=True,
        text=True,
    )

    build_time = time.time() - build_start

    if result.returncode == 0:
        print(f"✓ Build completed in {build_time:.1f}s")
        return True, "", build_time
    else:
        print(f"❌ Build failed:\n{result.stderr}")
        return False, result.stderr, build_time


def setup_weights_from_volume(workspace: Path) -> int:
    """
    Copy trained weights from persistent volume to workspace data/weights.
    
    Args:
        workspace: Path to workspace
        
    Returns:
        Number of weight files copied
    """
    import shutil

    print("\n📂 Setting up weights...")

    weights_dest = workspace / "data" / "weights"
    specialists_dest = weights_dest / "specialists"
    specialists_dest.mkdir(parents=True, exist_ok=True)

    trained_weights = Path("/experiments/trained_weights")
    weights_found = 0

    if not trained_weights.exists():
        print("  ⚠️  No trained weights found in volume")
        return 0

    # Copy generalist weights
    gen_weights = trained_weights / "generalist.toml"
    if gen_weights.exists():
        shutil.copy2(gen_weights, weights_dest / "generalist.toml")
        weights_found += 1

    # Copy specialist weights
    spec_dir = trained_weights / "specialists"
    if spec_dir.exists():
        for spec_file in spec_dir.glob("*.toml"):
            shutil.copy2(spec_file, specialists_dest / spec_file.name)
            weights_found += 1

    print(f"  Found {weights_found} weight files")
    return weights_found


def print_banner(text: str, char: str = "=", width: int = 70):
    """Print a formatted banner."""
    print("\n" + char * width)
    print(text)
    print(char * width)


# ============================================================================
# Training Function
# ============================================================================

@app.function(
    image=rust_image,
    cpu=CPU_COUNT,
    memory=MEMORY_MB,
    timeout=TIMEOUT_SECONDS,
    volumes={"/experiments": volume},
    )
def run_training(config: dict, workspace_snapshot: bytes):
    """
    Run a single training configuration on dedicated CPU instance.
    
    Args:
        config: Training configuration dict with tag, mode, args, description
        workspace_snapshot: Tarball of workspace directory
    """
    import shutil
    import subprocess
    import time

    print_banner(f"🚀 Starting: {config['description']}")
    print(f"📊 Tag: {config['tag']}")
    print(f"🔧 Mode: {config['mode']}")
    print(f"⚙️  Args: {' '.join(config['args'])}")
    print(f"💻 Resources: {CPU_COUNT} cores, {MEMORY_MB}MB RAM")
    print("=" * 70)

    start_time = time.time()

    # Extract workspace snapshot
    workspace_path = extract_workspace(workspace_snapshot)

    # Build in release mode
    success, stderr, build_time = build_rust_binary(workspace_path, "tune")
    if not success:
        raise RuntimeError(f"Build failed for {config['tag']}: {stderr[-500:]}")

    # Run tuning
    print("\n🧠 Starting training...")
    training_start = time.time()

    cmd = [
        "cargo", "run", "--release", "--bin", "tune", "--",
        f"--tag={config['tag']}",
        f"--mode={config['mode']}",
    ] + config['args']

    print(f"Command: {' '.join(cmd)}")

    result = subprocess.run(
        cmd,
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )

    training_time = time.time() - training_start

    if result.returncode != 0:
        print(f"❌ Training failed:\n{result.stderr}")
        raise RuntimeError(f"Training failed for {config['tag']}")

    # Parse results from output
    output = result.stdout
    print_banner("📈 TRAINING RESULTS")

    # Extract key metrics
    best_wr = None
    best_fitness = None
    generations = None

    for line in output.split('\n'):
        if "Best win rate:" in line:
            best_wr = line.split("Best win rate:")[-1].strip()
        elif "Best fitness:" in line:
            best_fitness = line.split("Best fitness:")[-1].strip()
        elif "Generations:" in line:
            generations = line.split("Generations:")[-1].strip()
        elif "Auto-deployed to" in line:
            deploy_path = line.split('"')[1]
            print(f"✓ Auto-deployed: {deploy_path}")

    if best_wr:
        print(f"🎯 Win Rate: {best_wr}")
    if best_fitness:
        print(f"📊 Fitness: {best_fitness}")
    if generations:
        print(f"🔄 Generations: {generations}")

    print(f"⏱️  Training Time: {training_time:.1f}s ({training_time/60:.1f}m)")

    # Copy experiment outputs to persistent volume
    print("\n💾 Saving results to persistent volume...")

    experiments_dir = workspace_path / "experiments" / "mcts"
    if experiments_dir.exists():
        # Find the experiment directory (timestamped)
        experiment_dirs = sorted(experiments_dir.glob("*"))
        if experiment_dirs:
            latest_exp = experiment_dirs[-1]
            dest_path = Path("/experiments") / latest_exp.name

            shutil.copytree(latest_exp, dest_path, dirs_exist_ok=True)
            print(f"✓ Saved to: {dest_path}")

            # Commit volume changes
            volume.commit()
            print("✓ Volume committed")

    # Save ONLY the trained weights to a consolidated location (not per-config)
    # This ensures we don't overwrite good weights with stale ones
    weights_dir = workspace_path / "data" / "weights"
    weights_dest = Path("/experiments") / "trained_weights"
    weights_dest.mkdir(parents=True, exist_ok=True)
    specialists_dest = weights_dest / "specialists"
    specialists_dest.mkdir(parents=True, exist_ok=True)

    if config['mode'] == 'generalist':
        # Only save generalist.toml
        src = weights_dir / "generalist.toml"
        if src.exists():
            shutil.copy2(src, weights_dest / "generalist.toml")
            print("✓ Saved generalist weights to volume")
            volume.commit()
    elif config['mode'] == 'faction-specialist':
        # Only save the specific faction's specialist weights
        faction = None
        for i, arg in enumerate(config['args']):
            if arg == '--faction' and i + 1 < len(config['args']):
                faction = config['args'][i + 1]
                break
        if faction:
            src = weights_dir / "specialists" / f"{faction}.toml"
            if src.exists():
                shutil.copy2(src, specialists_dest / f"{faction}.toml")
                print(f"✓ Saved {faction} specialist weights to volume")
                volume.commit()

    total_time = time.time() - start_time

    print_banner(f"✅ {config['description']} COMPLETE!")
    print(f"⏱️  Total Time: {total_time:.1f}s ({total_time/60:.1f}m)")
    print("=" * 70 + "\n")

    # Extract faction name for display (if applicable)
    display_name = config['mode']
    if config['mode'] == 'faction-specialist':
        for i, arg in enumerate(config['args']):
            if arg == '--faction' and i + 1 < len(config['args']):
                faction = config['args'][i + 1]
                display_name = faction.capitalize()
                break
    elif config['mode'] == 'generalist':
        display_name = 'generalist'

    return {
        "tag": config['tag'],
        "mode": config['mode'],
        "display_name": display_name,
        "best_wr": best_wr,
        "best_fitness": best_fitness,
        "training_time": training_time,
        "total_time": total_time,
        "success": True,
    }

# ============================================================================
# Validation Function
# ============================================================================

@app.function(
    image=rust_image,
    cpu=VALIDATION_CPU,
    memory=VALIDATION_MEMORY,
    timeout=VALIDATION_TIMEOUT,
    volumes={"/experiments": volume},
    )
def run_validation(workspace_snapshot: bytes, run_id: str = None, games: int = None, mcts_sims: int = None, cores: int = None):
    """
    Run balance validation with trained weights.

    Validates all faction matchups using round-robin deck testing (40 total matchups).
    Each matchup tests both player orders for comprehensive balance analysis.

    Args:
        workspace_snapshot: Tarball of workspace directory
        run_id: Optional run identifier for naming output
        games: Number of games per matchup per direction (total = 40 × 2 × games)
        mcts_sims: MCTS simulations per move (overrides VALIDATION_MCTS_SIMS)
        cores: Number of CPU cores (for display only, set via decorator)

    Returns:
        Dict with validation results
    """
    import json
    import subprocess
    import time

    # Use passed parameters or fall back to module defaults
    games = games or VALIDATION_GAMES
    mcts_sims = mcts_sims or VALIDATION_MCTS_SIMS
    cores = cores or VALIDATION_CPU

    print_banner("🔍 BALANCE VALIDATION")

    print_banner("🔍 BALANCE VALIDATION")
    print(f"⚙️  Games: {games}, MCTS sims: {mcts_sims}, Cores: {cores}")

    start_time = time.time()

    # Extract workspace snapshot
    workspace_path = extract_workspace(workspace_snapshot)

    # Build validate binary
    success, stderr, build_time = build_rust_binary(workspace_path, "validate")
    if not success:
        return {"success": False, "error": f"Build failed: {stderr[-1000:]}"}

    # Set up weights from persistent volume
    weights_found = setup_weights_from_volume(workspace_path)

    # Run validation
    print("\n🧪 Running validation...")
    print(f"   Games per matchup: {games}")
    print(f"   MCTS simulations: {mcts_sims}")

    validation_start = time.time()

    output_file = workspace_path / "validation_results.json"

    cmd = [
        str(workspace_path / "target/release/validate"),
        "--games", str(games),
        "--mcts-sims", str(mcts_sims),
        "--output", str(output_file),
    ]

    print(f"   Command: {' '.join(cmd)}")

    result = subprocess.run(
        cmd,
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )

    validation_time = time.time() - validation_start

    # Print validation output
    if result.stdout:
        print("\n" + result.stdout)

    # Load results
    validation_data = None
    if output_file.exists():
        with open(output_file) as f:
            validation_data = json.load(f)

    # Copy results to volume
    if output_file.exists():
        run_name = run_id or datetime.now().strftime("%Y-%m-%d_%H%M")
        dest_file = Path("/experiments") / f"validation_{run_name}.json"
        import shutil
        shutil.copy2(output_file, dest_file)
        print(f"\n💾 Results saved to: {dest_file}")
        volume.commit()

    total_time = time.time() - start_time

    print_banner("✅ VALIDATION COMPLETE!")
    print(f"⏱️  Validation Time: {validation_time:.1f}s ({validation_time/60:.1f}m)")
    print(f"⏱️  Total Time: {total_time:.1f}s")
    print("=" * 70 + "\n")

    return {
        "success": result.returncode == 0,
        "results": validation_data,
        "validation_time": validation_time,
        "total_time": total_time,
        "stdout": result.stdout,
        "stderr": result.stderr if result.returncode != 0 else "",
    }


# ============================================================================
# Parallel Matchup Validation Function
# ============================================================================

# Define all matchups for parallel validation
MATCHUPS = [
    "argentum-symbiote",
    "argentum-obsidion",
    "symbiote-obsidion",
]


@app.function(
    image=rust_image,
    cpu=VALIDATION_CPU,
    memory=VALIDATION_MEMORY,
    timeout=VALIDATION_TIMEOUT,
    volumes={"/experiments": volume},
)
def run_matchup_validation(
    matchup: str,
    workspace_snapshot: bytes,
    run_id: str,
    games: int,
    mcts_sims: int,
    cores: int,
):
    """
    Run validation for a single matchup in parallel.

    Part of parallel validation system - each matchup runs in its own container.
    Uses round-robin deck testing (all deck combinations within faction matchup).

    Args:
        matchup: Matchup identifier (e.g., "argentum-symbiote")
        workspace_snapshot: Tarball of workspace directory
        run_id: Run identifier for naming output
        games: Number of games per deck matchup per direction
        mcts_sims: MCTS simulations per move
        cores: Number of CPU cores (for display)

    Returns:
        Dict with matchup results
    """
    import json
    import subprocess
    import time

    print_banner(f"🎯 MATCHUP VALIDATION: {matchup.upper()}")
    print(f"⚙️  Games: {games}, MCTS sims: {mcts_sims}, Cores: {cores}")

    start_time = time.time()

    # Extract workspace snapshot
    workspace_path = extract_workspace(workspace_snapshot)

    # Build validate binary
    success, stderr, build_time = build_rust_binary(workspace_path, "validate")
    if not success:
        return {"success": False, "matchup": matchup, "error": f"Build failed: {stderr[-1000:]}"}

    # Set up weights from persistent volume
    weights_found = setup_weights_from_volume(workspace_path)

    # Run validation for this matchup only
    print(f"\n🧪 Running validation for {matchup}...")
    validation_start = time.time()

    output_file = workspace_path / f"validation_{matchup}.json"

    cmd = [
        str(workspace_path / "target/release/validate"),
        "--games", str(games),
        "--mcts-sims", str(mcts_sims),
        "--matchup", matchup,
        "--output", str(output_file),
    ]

    print(f"   Command: {' '.join(cmd)}")

    result = subprocess.run(
        cmd,
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )

    validation_time = time.time() - validation_start

    # Print validation output
    if result.stdout:
        print("\n" + result.stdout)

    # Load results
    validation_data = None
    has_valid_data = False
    if output_file.exists():
        with open(output_file) as f:
            validation_data = json.load(f)
            # Check if we got actual matchup data
            has_valid_data = bool(validation_data and validation_data.get("matchups"))

    total_time = time.time() - start_time

    # Determine success: we succeeded if we got valid data
    # (exit code 1 just means "imbalanced", which is still valid data)
    success = has_valid_data

    print_banner(f"✅ {matchup.upper()} COMPLETE!")
    print(f"⏱️  Validation Time: {validation_time:.1f}s ({validation_time/60:.1f}m)")
    print("=" * 70 + "\n")

    return {
        "success": success,
        "matchup": matchup,
        "results": validation_data,
        "validation_time": validation_time,
        "total_time": total_time,
        "balance_status": validation_data.get("summary", {}).get("overall_status", "unknown") if validation_data else "unknown",
        "stdout": result.stdout,
        "stderr": result.stderr if result.returncode != 0 and not has_valid_data else "",
    }


def merge_matchup_results(matchup_results: list, run_id: str) -> dict:
    """
    Merge individual matchup results into a combined validation result.

    Args:
        matchup_results: List of individual matchup result dicts
        run_id: Run identifier

    Returns:
        Combined validation results dict
    """
    from datetime import datetime

    # Collect all matchup data
    all_matchups = []
    total_validation_time = 0
    failed_matchups = []

    for result in matchup_results:
        matchup_name = result.get("matchup", "unknown")

        if not result.get("success"):
            failed_matchups.append(matchup_name)
            # Still try to get data even if marked as failed
            data = result.get("results", {})
            if data and "matchups" in data:
                all_matchups.extend(data["matchups"])
            continue

        data = result.get("results", {})
        if data and "matchups" in data:
            all_matchups.extend(data["matchups"])
        total_validation_time = max(total_validation_time, result.get("validation_time", 0))

    if not all_matchups:
        return {"success": False, "error": f"No matchup results collected. Failed: {failed_matchups}"}

    # Calculate combined summary
    faction_wins = {}
    faction_games = {}
    total_p1_wins = 0
    total_games = 0
    total_draws = 0

    for m in all_matchups:
        f1 = m["faction1"]
        f2 = m["faction2"]

        faction_wins[f1] = faction_wins.get(f1, 0) + m["faction1_total_wins"]
        faction_wins[f2] = faction_wins.get(f2, 0) + m["faction2_total_wins"]
        faction_games[f1] = faction_games.get(f1, 0) + m["total_games"] - m["draws"]
        faction_games[f2] = faction_games.get(f2, 0) + m["total_games"] - m["draws"]

        total_p1_wins += m["f1_as_p1_wins"]
        total_p1_wins += m["total_games"] // 2 - m["f1_as_p2_wins"] - m["draws"] // 2
        total_games += m["total_games"]
        total_draws += m["draws"]

    decisive_games = total_games - total_draws
    p1_win_rate = total_p1_wins / decisive_games if decisive_games > 0 else 0.5

    faction_win_rates = {
        f: wins / faction_games.get(f, 1)
        for f, wins in faction_wins.items()
    }

    rates = list(faction_win_rates.values())
    max_faction_delta = max(rates) - min(rates) if len(rates) >= 2 else 0

    # Determine status
    warnings = []

    if not (0.50 <= p1_win_rate <= 0.55):
        warnings.append(f"P1 win rate {p1_win_rate*100:.1f}% is outside ideal range (50-55%)")

    if max_faction_delta >= 0.15:
        warnings.append(f"Faction delta {max_faction_delta*100:.1f}% indicates significant imbalance")

    p1_status = "balanced" if 0.50 <= p1_win_rate <= 0.55 else ("warning" if 0.45 <= p1_win_rate <= 0.60 else "imbalanced")
    faction_status = "balanced" if max_faction_delta < 0.10 else ("warning" if max_faction_delta < 0.15 else "imbalanced")
    overall_status = "imbalanced" if "imbalanced" in [p1_status, faction_status] else ("warning" if "warning" in [p1_status, faction_status] else "balanced")

    # Build combined result
    combined = {
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "run_id": run_id,
        "parallel_execution": True,
        "matchups": all_matchups,
        "summary": {
            "p1_win_rate": p1_win_rate,
            "p1_status": p1_status,
            "faction_win_rates": faction_win_rates,
            "max_faction_delta": max_faction_delta,
            "faction_status": faction_status,
            "overall_status": overall_status,
            "warnings": warnings,
        },
        "timing": {
            "wall_time_seconds": total_validation_time,
            "matchup_times": {r["matchup"]: r["validation_time"] for r in matchup_results if r.get("validation_time")},
        },
    }

    # Success = we got all matchup data (3 matchups expected)
    # Balance status (balanced/imbalanced) is separate from execution success
    data_complete = len(all_matchups) == 3
    return {"success": data_complete, "results": combined, "balance_status": overall_status}


# ============================================================================
# Download Results Function
# ============================================================================

@app.function(
    image=rust_image,
    volumes={"/experiments": volume},
)
def list_experiments():
    """List all experiments stored in persistent volume."""

    exp_path = Path("/experiments")
    if not exp_path.exists():
        return []

    experiments = []
    for item in exp_path.iterdir():
        if item.is_dir() and item.name != "weights":
            stat = item.stat()
            experiments.append({
                "name": item.name,
                "size_mb": sum(f.stat().st_size for f in item.rglob("*") if f.is_file()) / (1024 * 1024),
                "modified": stat.st_mtime,
            })

    return sorted(experiments, key=lambda x: x['modified'], reverse=True)

@app.function(
    image=rust_image,
    volumes={"/experiments": volume},
)
def download_experiment(experiment_name: str) -> bytes:
    """Download experiment results as tarball."""
    import io
    import tarfile

    exp_path = Path("/experiments") / experiment_name
    if not exp_path.exists():
        raise ValueError(f"Experiment {experiment_name} not found")

    # Create tarball in memory
    tarball = io.BytesIO()
    with tarfile.open(fileobj=tarball, mode='w:gz') as tar:
        tar.add(exp_path, arcname=experiment_name)

    tarball.seek(0)
    return tarball.read()


@app.function(
    image=rust_image,
    volumes={"/experiments": volume},
)
def download_trained_weights() -> dict:
    """
    Download trained weights from the Modal volume.

    Returns:
        Dict with weight file names and their contents
    """
    # Use consolidated weights location
    weights_path = Path("/experiments/trained_weights")
    weights = {}

    if not weights_path.exists():
        return weights

    # Copy generalist weights
    gen_file = weights_path / "generalist.toml"
    if gen_file.exists():
        weights["generalist.toml"] = gen_file.read_text()

    # Copy specialist weights
    spec_dir = weights_path / "specialists"
    if spec_dir.exists():
        for spec_file in spec_dir.glob("*.toml"):
            key = f"specialists/{spec_file.name}"
            weights[key] = spec_file.read_text()

    return weights


def deploy_weights_locally(weights: dict, workspace_path: Path) -> int:
    """
    Deploy downloaded weights to the local data/weights/ directory.

    Args:
        weights: Dict of filename -> content
        workspace_path: Path to the workspace root

    Returns:
        Number of weight files deployed
    """
    weights_dir = workspace_path / "data" / "weights"
    specialists_dir = weights_dir / "specialists"
    specialists_dir.mkdir(parents=True, exist_ok=True)

    deployed = 0
    for filename, content in weights.items():
        if filename.startswith("specialists/"):
            dest = weights_dir / filename
        else:
            dest = weights_dir / filename

        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(content)
        deployed += 1

    return deployed


# ============================================================================
# Local Utilities
# ============================================================================

def save_validation_results(workspace_path: Path, run_id: str, results: dict) -> Path:
    """
    Save validation results to the local experiments/validation/ directory.

    Args:
        workspace_path: Path to the workspace root
        run_id: Run identifier (e.g., "2026-01-15_0747")
        results: Validation results dict

    Returns:
        Path to the saved results directory
    """
    import json

    # Create validation results directory
    validation_dir = workspace_path / "experiments" / "validation" / run_id
    validation_dir.mkdir(parents=True, exist_ok=True)

    # Save full JSON results
    results_file = validation_dir / "results.json"
    with open(results_file, "w") as f:
        json.dump(results, f, indent=2)

    # Create human-readable summary
    summary = results.get("summary", {})
    summary_lines = [
        "=" * 60,
        f"VALIDATION SUMMARY - {run_id}",
        "=" * 60,
        "",
        f"Timestamp: {results.get('timestamp', 'N/A')}",
        f"Parallel Execution: {results.get('parallel_execution', False)}",
        "",
        "--- Balance Status ---",
        f"Overall Status: {summary.get('overall_status', 'unknown').upper()}",
        f"P1 Win Rate: {summary.get('p1_win_rate', 0) * 100:.1f}%",
        f"Max Faction Delta: {summary.get('max_faction_delta', 0) * 100:.1f}%",
        "",
        "--- Faction Win Rates ---",
    ]

    for faction, rate in summary.get("faction_win_rates", {}).items():
        summary_lines.append(f"  {faction.capitalize()}: {rate * 100:.1f}%")

    if warnings := summary.get("warnings", []):
        summary_lines.append("")
        summary_lines.append("--- Warnings ---")
        for w in warnings:
            summary_lines.append(f"  - {w}")

    # Add timing info if available
    timing = results.get("timing", {})
    if timing:
        summary_lines.append("")
        summary_lines.append("--- Timing ---")
        summary_lines.append(f"Wall Time: {timing.get('wall_time_seconds', 0):.1f}s")
        matchup_times = timing.get("matchup_times", {})
        if matchup_times:
            for matchup, t in matchup_times.items():
                summary_lines.append(f"  {matchup}: {t:.1f}s")

    # Add matchup details
    summary_lines.append("")
    summary_lines.append("--- Matchup Details ---")
    for m in results.get("matchups", []):
        f1 = m.get("faction1", "?").capitalize()
        f2 = m.get("faction2", "?").capitalize()
        f1_wr = m.get("faction1_win_rate", 0) * 100
        f2_wr = m.get("faction2_win_rate", 0) * 100
        summary_lines.append(f"{f1} vs {f2}: {f1_wr:.1f}% / {f2_wr:.1f}%")

    summary_lines.append("")
    summary_lines.append("=" * 60)

    summary_file = validation_dir / "summary.txt"
    with open(summary_file, "w") as f:
        f.write("\n".join(summary_lines))

    # Save config as TOML
    config_lines = [
        "# Validation Run Configuration",
        f'run_id = "{run_id}"',
        f'timestamp = "{results.get("timestamp", "")}"',
        f'parallel_execution = {str(results.get("parallel_execution", False)).lower()}',
        "",
        "[summary]",
        f'overall_status = "{summary.get("overall_status", "unknown")}"',
        f"p1_win_rate = {summary.get('p1_win_rate', 0):.4f}",
        f"max_faction_delta = {summary.get('max_faction_delta', 0):.4f}",
    ]

    config_file = validation_dir / "config.toml"
    with open(config_file, "w") as f:
        f.write("\n".join(config_lines))

    print(f"\n💾 Validation results saved to: {validation_dir}")
    print("   📄 results.json - Full JSON data")
    print("   📝 summary.txt  - Human-readable summary")
    print("   ⚙️  config.toml  - Run configuration")

    return validation_dir


# ============================================================================
# Local Entry Points
# ============================================================================

@app.local_entrypoint()
def main(
    single: str = None,
    mode: str = "full",
    no_deploy: bool = False,
    validation_games: int = None,
    validation_timeout: int = None,
    cores: int = None,
    sequential: bool = False,
):
    """
    Run training on Modal.

    Modes:
    - full (default): Train all configs + validate + deploy weights
    - train-only: Train only (skip validation) + deploy weights
    - validate-only: Skip training, only validate with existing weights

    Validation uses round-robin testing: 40 deck matchups × 2 directions × games.
    Default 500 games = 40,000 total games tested.

    Args:
        single: Run single configuration (e.g., 'generalist', 'argentum', 'symbiote', 'obsidion')
        mode: Pipeline mode - 'full' (train+validate), 'train-only', or 'validate-only'
        no_deploy: If True, skip auto-deploying weights to local data/weights/
        validation_games: Games per matchup per direction (total = 40 × 2 × games, default: 500)
        validation_timeout: Timeout in seconds (default: 3600)
        cores: Number of CPU cores for validation (default: 32, note: also update VALIDATION_CPU constant)
        sequential: If True, run validation sequentially instead of parallel (default: False)
    """
    import io
    import tarfile
    import time
    from datetime import datetime

    run_id = datetime.now().strftime("%Y-%m-%d_%H%M")

    # Apply defaults for optional parameters
    val_games = validation_games or VALIDATION_GAMES
    val_timeout = validation_timeout or VALIDATION_TIMEOUT
    val_cores = cores or VALIDATION_CPU

    parallel_mode = not sequential

    print_banner("🚀 ESSENCE WARS - MODAL CLOUD TRAINING")
    print(f"⏰ Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"🆔 Run ID: {run_id}")
    print(f"📋 Mode: {mode}")
    print(f"💻 Training: {CPU_COUNT} cores per job, {MEMORY_MB}MB RAM")
    print(f"💻 Validation: {val_cores} cores, {val_games} games/matchup")
    print(f"🔀 Validation Mode: {'PARALLEL (3 containers)' if parallel_mode else 'SEQUENTIAL (1 container)'}")
    print(f"⏱️  Timeout: Training {TIMEOUT_SECONDS}s, Validation {val_timeout}s")
    print("=" * 70 + "\n")

    # Create workspace snapshot (exclude target/, experiments/, .git/)
    print("📦 Creating workspace snapshot...")
    workspace_path = Path(__file__).parent

    snapshot = io.BytesIO()
    with tarfile.open(fileobj=snapshot, mode='w:gz') as tar:
        for item in workspace_path.iterdir():
            if item.name not in ['target', 'experiments', '.git', 'python', '__pycache__']:
                tar.add(item, arcname=item.name)

    snapshot.seek(0)
    workspace_snapshot = snapshot.read()

    print(f"✓ Workspace snapshot: {len(workspace_snapshot) / (1024*1024):.1f} MB\n")

    total_start_time = time.time()
    training_results = []

    # ========================================================================
    # Phase 1: Training
    # ========================================================================
    if mode in ("full", "train-only"):
        print("=" * 70)
        print("📚 PHASE 1: TRAINING")
        print("=" * 70 + "\n")

        # Select configurations to run
        if single:
            configs = [c for c in TRAINING_CONFIGS if single.lower() in c['tag'].lower()]
            if not configs:
                print(f"❌ No configuration found matching '{single}'")
                print("Available: generalist, argentum, symbiote, obsidion")
                sys.exit(1)
            print(f"🎯 Running single configuration: {configs[0]['description']}\n")
        else:
            configs = TRAINING_CONFIGS
            print(f"🎯 Running {len(configs)} configurations in PARALLEL\n")

        # Run training jobs in parallel
        training_start = time.time()

        print("🚀 Launching training jobs...\n")
        training_results = list(run_training.map(configs, [workspace_snapshot] * len(configs)))

        training_time = time.time() - training_start

        # Print training summary
        print_banner("📊 TRAINING SUMMARY")

        success_count = sum(1 for r in training_results if r['success'])

        print(f"\n✅ Completed: {success_count}/{len(configs)}")
        print(f"⏱️  Training Time: {training_time:.1f}s ({training_time/60:.1f}m)")
        if training_time > 0:
            print(f"⚡ Speedup: {sum(r.get('training_time', 0) for r in training_results) / training_time:.1f}x (vs sequential)\n")

        print("┌─────────────────────────────────────────────────────────────────┐")
        print("│                    TRAINING RESULTS                             │")
        print("├─────────────────────────────────────────────────────────────────┤")

        for result in training_results:
            if result.get('success'):
                wr = result.get('best_wr') or 'N/A'
                fit = result.get('best_fitness') or 'N/A'
                time_m = result.get('training_time', 0) / 60
                display_name = result.get('display_name', result['mode'])
                print(f"│ {display_name:20s} │ WR: {wr:>6s} │ Fit: {fit:>7s} │ {time_m:4.1f}m │")

        print("└─────────────────────────────────────────────────────────────────┘\n")

        # Auto-deploy weights to local repository
        if not no_deploy and any(r.get('success') for r in training_results):
            print("=" * 70)
            print("📦 DEPLOYING WEIGHTS")
            print("=" * 70 + "\n")

            print("⬇️  Downloading trained weights from Modal...")
            weights = download_trained_weights.remote()

            if weights:
                deployed = deploy_weights_locally(weights, workspace_path)
                print(f"✅ Deployed {deployed} weight file(s) to data/weights/")
                for filename in sorted(weights.keys()):
                    print(f"   → {filename}")
                print()
            else:
                print("⚠️  No weights found in Modal volume to deploy\n")

        if mode == "train-only":
            print("💡 Training complete. Run with --mode full or --mode validate-only to run validation.\n")

    # ========================================================================
    # Phase 2: Validation
    # ========================================================================
    validation_result = None

    if mode in ("full", "validate-only"):
        print("=" * 70)
        print("🔍 PHASE 2: VALIDATION")
        print("=" * 70 + "\n")

        validation_start = time.time()

        if parallel_mode:
            # Run matchups in parallel (3 containers)
            print(f"🚀 Launching {len(MATCHUPS)} parallel validation workers...")
            print(f"   Games per matchup: {val_games}")
            print(f"   Cores per worker: {val_cores}")
            print()

            # Launch all matchup validations in parallel
            matchup_results = list(run_matchup_validation.map(
                MATCHUPS,
                [workspace_snapshot] * len(MATCHUPS),
                [run_id] * len(MATCHUPS),
                [val_games] * len(MATCHUPS),
                [VALIDATION_MCTS_SIMS] * len(MATCHUPS),
                [val_cores] * len(MATCHUPS),
            ))

            # Merge results
            validation_result = merge_matchup_results(matchup_results, run_id)

            # Print individual matchup times
            if validation_result.get("success"):
                timing = validation_result.get("results", {}).get("timing", {})
                matchup_times = timing.get("matchup_times", {})
                if matchup_times:
                    print("\n⏱️  Individual Matchup Times:")
                    for matchup, t in matchup_times.items():
                        print(f"   {matchup}: {t:.1f}s ({t/60:.1f}m)")
        else:
            # Run sequentially (original behavior)
            print(f"🎯 Running balance validation sequentially ({val_games} games/matchup, {val_cores} cores)...\n")

            seq_result = run_validation.remote(
                workspace_snapshot,
                run_id,
                games=val_games,
                mcts_sims=VALIDATION_MCTS_SIMS,
                cores=val_cores,
            )
            validation_result = {
                "success": seq_result.get("success"),
                "results": seq_result.get("results"),
            }

        validation_time = time.time() - validation_start

        # Check if we have results to display/save
        results = validation_result.get('results', {}) if validation_result else {}
        has_results = bool(results and results.get('matchups'))

        if has_results:
            # Print validation summary
            summary = results.get('summary', {})

            print_banner("📊 VALIDATION SUMMARY")

            p1_wr = summary.get('p1_win_rate', 0) * 100
            status = summary.get('overall_status', 'unknown').upper()

            print(f"\n🎯 P1 Win Rate: {p1_wr:.1f}%")
            print(f"📋 Overall Status: {status}")

            print("\n📊 Faction Win Rates:")
            for faction, rate in summary.get('faction_win_rates', {}).items():
                print(f"   {faction.capitalize()}: {rate*100:.1f}%")

            max_delta = summary.get('max_faction_delta', 0) * 100
            print(f"\n📏 Max Faction Delta: {max_delta:.1f}%")

            if warnings := summary.get('warnings', []):
                print("\n⚠️  Warnings:")
                for w in warnings:
                    print(f"   - {w}")

            print(f"\n⏱️  Validation Time: {validation_time:.1f}s ({validation_time/60:.1f}m)")
            if parallel_mode:
                # Calculate speedup
                matchup_times = results.get("timing", {}).get("matchup_times", {})
                if matchup_times:
                    sequential_time = sum(matchup_times.values())
                    speedup = sequential_time / validation_time if validation_time > 0 else 1
                    print(f"⚡ Parallel Speedup: {speedup:.1f}x vs sequential")

            print()

            # Always save results locally when we have data
            save_validation_results(workspace_path, run_id, results)
        else:
            error_msg = validation_result.get('error', 'Unknown') if validation_result else 'No result'
            print(f"❌ Validation failed: {error_msg}")

    # ========================================================================
    # Final Summary
    # ========================================================================
    total_time = time.time() - total_start_time

    print_banner("📊 FINAL SUMMARY")

    print(f"\n⏱️  Total Wall Time: {total_time:.1f}s ({total_time/60:.1f}m)")

    # Show deployment status
    if mode in ("full", "train-only") and training_results:
        if no_deploy:
            print("\n📦 Weights NOT auto-deployed (--no-deploy flag used)")
            print("   To deploy manually: modal run modal_tune.py::download_latest")
        else:
            print("\n✅ Weights auto-deployed to data/weights/")

    # Instructions for additional downloads
    print("\n📥 ADDITIONAL DOWNLOADS:")
    print("   modal run modal_tune.py::list_experiments")
    print("   modal run modal_tune.py::download_latest\n")

    print("💡 TIP: Full results are stored in Modal's persistent volume.")
    print("   View in Modal dashboard: https://modal.com/storage\n")

    # Estimate cost
    if training_results:
        num_training_jobs = len([r for r in training_results if r.get('success')])
        training_cpu_hours = sum(r.get('training_time', 0) for r in training_results) / 3600 * CPU_COUNT
    else:
        num_training_jobs = 0
        training_cpu_hours = 0

    validation_cpu_hours = (VALIDATION_TIMEOUT / 3600) * VALIDATION_CPU * 0.3  # Estimate 30% of timeout
    total_cpu_hours = training_cpu_hours + validation_cpu_hours
    estimated_cost = total_cpu_hours * 0.025  # ~$0.025 per CPU-hour

    print(f"💰 Estimated Cost: ${estimated_cost:.2f}")
    print(f"   ({total_cpu_hours:.1f} CPU-hours at ~$0.025/CPU-hour)\n")

    print("=" * 70)
    print("✅ ALL DONE!")
    print("=" * 70 + "\n")

@app.local_entrypoint()
def download_latest():
    """Download the latest experiment results."""

    print("📋 Fetching experiment list...")
    experiments = list_experiments.remote()

    if not experiments:
        print("❌ No experiments found in persistent volume")
        return

    print(f"\n📦 Found {len(experiments)} experiment(s):")
    for i, exp in enumerate(experiments[:5]):  # Show latest 5
        print(f"  {i+1}. {exp['name']} ({exp['size_mb']:.1f} MB)")

    # Download latest
    latest = experiments[0]
    print(f"\n⬇️  Downloading: {latest['name']}...")

    tarball = download_experiment.remote(latest['name'])

    output_path = Path(f"experiments_modal_{latest['name']}.tar.gz")
    output_path.write_bytes(tarball)

    print(f"✅ Downloaded to: {output_path}")
    print(f"📦 Size: {len(tarball) / (1024*1024):.1f} MB")
    print(f"\n💡 Extract with: tar -xzf {output_path}")
