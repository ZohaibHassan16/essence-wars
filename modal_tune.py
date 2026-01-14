#!/usr/bin/env python3
"""
Modal Cloud Training for Essence Wars
Runs all 4 tuning configurations in parallel on dedicated CPU instances.

Usage:
    modal run modal_tune.py                    # Run all 4 trainings in parallel
    modal run modal_tune.py --single generalist # Run single training
    modal deploy modal_tune.py                 # Deploy as persistent app
"""

import modal
import sys
from pathlib import Path

# ============================================================================
# Configuration
# ============================================================================

APP_NAME = "essence-wars-tuning"
RUST_VERSION = "1.75.0"  # Stable Rust version

# CPU configuration (scale up for faster training)
CPU_COUNT = 16  # 16 cores per training job (excellent parallel performance)
MEMORY_MB = 8192  # 8 GB RAM (sufficient for MCTS)
TIMEOUT_SECONDS = 7200  # 2 hours max (usually finishes in 10-15 min)

# Training configurations
TRAINING_CONFIGS = [
    {
        "tag": "generalist-v0.4-modal",
        "mode": "generalist",
        "args": ["--generations", "100", "--games", "100", "--mcts-sims", "50"],
        "description": "Universal weights for all decks",
    },
    {
        "tag": "argentum-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "argentum", "--generations", "100", "--games", "100", "--mcts-sims", "50"],
        "description": "Argentum (defensive control) specialist",
    },
    {
        "tag": "symbiote-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "symbiote", "--generations", "100", "--games", "100", "--mcts-sims", "50"],
        "description": "Symbiote (aggressive tempo) specialist",
    },
    {
        "tag": "obsidion-specialist-v0.4-modal",
        "mode": "faction-specialist",
        "args": ["--faction", "obsidion", "--generations", "100", "--games", "100", "--mcts-sims", "50"],
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
    _allow_background_volume_commits=True,
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
    
    # Also save weights to volume
    weights_dir = workspace_path / "data" / "weights"
    if weights_dir.exists():
        weights_dest = Path("/experiments") / "weights" / config['tag']
        weights_dest.mkdir(parents=True, exist_ok=True)
        
        for weights_file in weights_dir.rglob("*.toml"):
            if weights_file.is_file():
                rel_path = weights_file.relative_to(weights_dir)
                dest_file = weights_dest / rel_path
                dest_file.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(weights_file, dest_file)
        
        print(f"✓ Weights saved to: {weights_dest}")
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

# ============================================================================
# Local Entry Points
# ============================================================================

@app.local_entrypoint()
def main(single: str = None):
    """
    Run training on Modal.
    
    Args:
        single: Run single configuration (e.g., 'generalist', 'argentum', 'symbiote', 'obsidion')
    """
    import tarfile
    import io
    import time
    from datetime import datetime
    
    print("\n" + "=" * 70)
    print("🚀 ESSENCE WARS - MODAL CLOUD TRAINING")
    print("=" * 70)
    print(f"⏰ Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
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
    start_time = time.time()
    
    print("🚀 Launching training jobs...\n")
    results = list(run_training.map(configs, [workspace_snapshot] * len(configs)))
    
    total_time = time.time() - start_time
    
    # Print summary
    print("\n" + "=" * 70)
    print("📊 FINAL SUMMARY")
    print("=" * 70)
    
    success_count = sum(1 for r in results if r['success'])
    
    print(f"\n✅ Completed: {success_count}/{len(configs)}")
    print(f"⏱️  Total Wall Time: {total_time:.1f}s ({total_time/60:.1f}m)")
    print(f"⚡ Speedup: {sum(r['training_time'] for r in results) / total_time:.1f}x (vs sequential)\n")
    
    print("┌─────────────────────────────────────────────────────────────────┐")
    print("│                         RESULTS                                 │")
    print("├─────────────────────────────────────────────────────────────────┤")
    
    for result in results:
        if result['success']:
            wr = result['best_wr'] or 'N/A'
            fit = result['best_fitness'] or 'N/A'
            time_m = result['training_time'] / 60
            print(f"│ {result['mode']:20s} │ WR: {wr:>6s} │ Fit: {fit:>7s} │ {time_m:4.1f}m │")
    
    print("└─────────────────────────────────────────────────────────────────┘\n")
    
    # Instructions for downloading results
    print("📥 DOWNLOAD RESULTS:")
    print("   modal run modal_tune.py::list_experiments")
    print("   modal run modal_tune.py::download_latest\n")
    
    print("💡 TIP: Results are stored in Modal's persistent volume.")
    print("   View in Modal dashboard: https://modal.com/storage\n")
    
    # Estimate cost
    total_cpu_hours = (total_time / 3600) * len(configs) * CPU_COUNT
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
