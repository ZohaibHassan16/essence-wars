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

import modal
import sys
from pathlib import Path

# ============================================================================
# Configuration
# ============================================================================

APP_NAME = "essence-wars-tuning"
RUST_VERSION = "stable"  # Use latest stable Rust

# CPU configuration (scale up for faster training)
CPU_COUNT = 16  # 16 cores per training job (excellent parallel performance)
MEMORY_MB = 8192  # 8 GB RAM (sufficient for MCTS)
TIMEOUT_SECONDS = 7200  # 2 hours max (usually finishes in 10-15 min)

# Validation configuration
VALIDATION_CPU = 16
VALIDATION_MEMORY = 16384  # 16 GB
VALIDATION_TIMEOUT = 3600  # 1 hour
VALIDATION_GAMES = 500  # Games per matchup
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
    import subprocess
    import tarfile
    import io
    import shutil
    import time
    from datetime import datetime
    
    print("=" * 70)
    print(f"🚀 Starting: {config['description']}")
    print(f"📊 Tag: {config['tag']}")
    print(f"🔧 Mode: {config['mode']}")
    print(f"⚙️  Args: {' '.join(config['args'])}")
    print(f"💻 Resources: {CPU_COUNT} cores, {MEMORY_MB}MB RAM")
    print("=" * 70)
    
    start_time = time.time()
    
    # Extract workspace snapshot
    print("\n📦 Extracting workspace...")
    workspace_path = Path("/tmp/essence-wars")
    workspace_path.mkdir(exist_ok=True)
    
    with tarfile.open(fileobj=io.BytesIO(workspace_snapshot), mode='r:gz') as tar:
        tar.extractall(workspace_path)
    
    print(f"✓ Workspace extracted to {workspace_path}")
    
    # Build in release mode
    print("\n🔨 Building release binary...")
    build_start = time.time()
    
    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "tune"],
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )
    
    build_time = time.time() - build_start
    print(f"✓ Build completed in {build_time:.1f}s")
    
    if result.returncode != 0:
        print(f"❌ Build failed:\n{result.stderr}")
        raise RuntimeError(f"Build failed for {config['tag']}")
    
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
    print("\n" + "=" * 70)
    print("📈 TRAINING RESULTS")
    print("=" * 70)
    
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
            print(f"✓ Saved generalist weights to volume")
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
    
    print("\n" + "=" * 70)
    print(f"✅ {config['description']} COMPLETE!")
    print(f"⏱️  Total Time: {total_time:.1f}s ({total_time/60:.1f}m)")
    print("=" * 70 + "\n")
    
    return {
        "tag": config['tag'],
        "mode": config['mode'],
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
def run_validation(workspace_snapshot: bytes, run_id: str = None):
    """
    Run balance validation with trained weights.

    Args:
        workspace_snapshot: Tarball of workspace directory
        run_id: Optional run identifier for naming output

    Returns:
        Dict with validation results
    """
    import subprocess
    import tarfile
    import io
    import json
    import time
    from datetime import datetime

    print("=" * 70)
    print("🔍 BALANCE VALIDATION")
    print("=" * 70)

    start_time = time.time()

    # Extract workspace snapshot
    print("\n📦 Extracting workspace...")
    workspace_path = Path("/tmp/essence-wars")
    workspace_path.mkdir(exist_ok=True)

    with tarfile.open(fileobj=io.BytesIO(workspace_snapshot), mode='r:gz') as tar:
        tar.extractall(workspace_path)

    print(f"✓ Workspace extracted to {workspace_path}")

    # Build validate binary
    print("\n🔨 Building validate binary...")
    build_start = time.time()

    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "validate"],
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )

    build_time = time.time() - build_start
    print(f"✓ Build completed in {build_time:.1f}s")

    if result.returncode != 0:
        print(f"❌ Build failed:\n{result.stderr}")
        return {"success": False, "error": f"Build failed: {result.stderr[-1000:]}"}

    # Set up weights - copy from consolidated training outputs
    print("\n📂 Setting up weights...")
    weights_dest = workspace_path / "data" / "weights"
    specialists_dest = weights_dest / "specialists"
    specialists_dest.mkdir(parents=True, exist_ok=True)

    # Look for trained weights in the consolidated location
    trained_weights = Path("/experiments/trained_weights")
    weights_found = 0

    if trained_weights.exists():
        import shutil

        # Copy generalist weights
        gen_weights = trained_weights / "generalist.toml"
        if gen_weights.exists():
            shutil.copy2(gen_weights, weights_dest / "generalist.toml")
            print(f"  ✓ Copied generalist weights")
            weights_found += 1

        # Copy specialist weights
        spec_dir = trained_weights / "specialists"
        if spec_dir.exists():
            for spec_file in spec_dir.glob("*.toml"):
                shutil.copy2(spec_file, specialists_dest / spec_file.name)
                print(f"  ✓ Copied {spec_file.stem} specialist weights")
                weights_found += 1

    print(f"  Found {weights_found} weight files")

    # Run validation
    print("\n🧪 Running validation...")
    print(f"   Games per matchup: {VALIDATION_GAMES}")
    print(f"   MCTS simulations: {VALIDATION_MCTS_SIMS}")

    validation_start = time.time()

    output_file = workspace_path / "validation_results.json"

    cmd = [
        str(workspace_path / "target/release/validate"),
        "--games", str(VALIDATION_GAMES),
        "--mcts-sims", str(VALIDATION_MCTS_SIMS),
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

    print("\n" + "=" * 70)
    print(f"✅ VALIDATION COMPLETE!")
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
# Download Results Function
# ============================================================================

@app.function(
    image=rust_image,
    volumes={"/experiments": volume},
)
def list_experiments():
    """List all experiments stored in persistent volume."""
    import os
    
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
    import tarfile
    import io

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
# Local Entry Points
# ============================================================================

@app.local_entrypoint()
def main(single: str = None, mode: str = "full", no_deploy: bool = False):
    """
    Run training on Modal.

    Args:
        single: Run single configuration (e.g., 'generalist', 'argentum', 'symbiote', 'obsidion')
        mode: Pipeline mode - 'full' (train+validate), 'train-only', or 'validate-only'
        no_deploy: If True, skip auto-deploying weights to local data/weights/
    """
    import tarfile
    import io
    import time
    from datetime import datetime

    run_id = datetime.now().strftime("%Y-%m-%d_%H%M")

    print("\n" + "=" * 70)
    print("🚀 ESSENCE WARS - MODAL CLOUD TRAINING")
    print("=" * 70)
    print(f"⏰ Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"🆔 Run ID: {run_id}")
    print(f"📋 Mode: {mode}")
    print(f"💻 Resources: {CPU_COUNT} cores per job, {MEMORY_MB}MB RAM")
    print(f"⏱️  Timeout: {TIMEOUT_SECONDS}s per job")
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
                print(f"Available: generalist, argentum, symbiote, obsidion")
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
        print("\n" + "=" * 70)
        print("📊 TRAINING SUMMARY")
        print("=" * 70)

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
                print(f"│ {result['mode']:20s} │ WR: {wr:>6s} │ Fit: {fit:>7s} │ {time_m:4.1f}m │")

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
    if mode in ("full", "validate-only"):
        print("=" * 70)
        print("🔍 PHASE 2: VALIDATION")
        print("=" * 70 + "\n")

        print(f"🎯 Running balance validation ({VALIDATION_GAMES} games/matchup)...\n")

        validation_result = run_validation.remote(workspace_snapshot, run_id)

        if validation_result.get('success'):
            # Print validation summary
            results = validation_result.get('results', {})
            summary = results.get('summary', {})

            print("\n" + "=" * 70)
            print("📊 VALIDATION SUMMARY")
            print("=" * 70)

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

            print()
        else:
            print(f"❌ Validation failed: {validation_result.get('error', validation_result.get('stderr', 'Unknown'))}")

    # ========================================================================
    # Final Summary
    # ========================================================================
    total_time = time.time() - total_start_time

    print("\n" + "=" * 70)
    print("📊 FINAL SUMMARY")
    print("=" * 70)

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
    import time
    
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
