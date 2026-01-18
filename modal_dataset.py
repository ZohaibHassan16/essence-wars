#!/usr/bin/env python3
"""
Modal Cloud Dataset Generation for Essence Wars

Generates large MCTS self-play datasets using cloud compute.
The Rust binary handles internal parallelization via rayon.

Usage:
    # Generate 100k games (quick test, ~5 min)
    modal run modal_dataset.py --games 100000

    # Generate 1M games (full dataset, ~45-60 min)
    modal run modal_dataset.py --games 1000000

    # Download dataset from Modal volume
    modal run modal_dataset.py --download

    # List existing datasets
    modal run modal_dataset.py --list
"""

from datetime import datetime
from pathlib import Path

import modal

# ============================================================================
# Configuration
# ============================================================================

APP_NAME = "essence-wars-dataset"
RUST_VERSION = "stable"

# Container configuration
CPU_COUNT = 16  # 16 cores for rayon parallelization
MEMORY_MB = 16384  # 16 GB RAM (increased for 100k+ datasets)
TIMEOUT_SECONDS = 14400  # 4 hours max

# Default generation parameters
DEFAULT_GAMES = 100_000
DEFAULT_SIMS = 100
DEFAULT_MODE = "self-play"

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
        f"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {RUST_VERSION}",
        "echo 'source $HOME/.cargo/env' >> ~/.bashrc",
    )
    .env({"PATH": "/root/.cargo/bin:$PATH"})
)

# Persistent volume for datasets
volume = modal.Volume.from_name("essence-wars-datasets", create_if_missing=True)
VOLUME_PATH = "/data/datasets"

# ============================================================================
# Helper Functions
# ============================================================================

def extract_workspace(workspace_snapshot: bytes, path: str = "/tmp/essence-wars") -> Path:
    """Extract workspace snapshot tarball to specified path."""
    import io
    import tarfile

    print("\n[1/4] Extracting workspace...")
    workspace_path = Path(path)
    workspace_path.mkdir(exist_ok=True)

    with tarfile.open(fileobj=io.BytesIO(workspace_snapshot), mode='r:gz') as tar:
        tar.extractall(workspace_path)

    print(f"  Workspace extracted to {workspace_path}")
    return workspace_path


def build_binary(workspace: Path) -> bool:
    """Build the generate_dataset binary."""
    import subprocess
    import time

    print("\n[2/4] Building generate_dataset binary...")
    build_start = time.time()

    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "generate_dataset"],
        cwd=workspace,
        capture_output=True,
        text=True,
    )

    build_time = time.time() - build_start

    if result.returncode == 0:
        print(f"  Build completed in {build_time:.1f}s")
        return True
    else:
        print(f"  Build failed:\n{result.stderr}")
        return False


def create_workspace_snapshot() -> bytes:
    """Create a gzipped tarball of the workspace for Modal upload."""
    import io
    import tarfile

    # Find workspace root (where Cargo.toml is)
    script_dir = Path(__file__).parent.absolute()
    workspace = script_dir

    # Verify we're in the right place
    if not (workspace / "Cargo.toml").exists():
        raise FileNotFoundError(f"Cargo.toml not found in {workspace}")

    print(f"Creating workspace snapshot from: {workspace}")

    # Create tarball in memory
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode='w:gz') as tar:
        for item in workspace.iterdir():
            # Skip files/directories we don't need
            skip_patterns = [
                'target', '.git', '__pycache__', '.venv', 'node_modules',
                'experiments', '.pytest_cache', '.mypy_cache', '.ruff_cache',
                '*.pyc', '.DS_Store',
            ]
            if any(item.name.startswith(p.replace('*', '')) or item.name == p for p in skip_patterns):
                continue
            if item.name.startswith('.') and item.name not in ['.cargo']:
                continue

            print(f"  Adding: {item.name}")
            tar.add(item, arcname=item.name)

    buffer.seek(0)
    snapshot = buffer.read()
    print(f"Snapshot size: {len(snapshot) / 1024 / 1024:.1f} MB")
    return snapshot


# ============================================================================
# Modal Functions
# ============================================================================

@app.function(
    image=rust_image,
    cpu=CPU_COUNT,
    memory=MEMORY_MB,
    timeout=TIMEOUT_SECONDS,
    volumes={VOLUME_PATH: volume},
)
def generate_dataset(
    workspace_snapshot: bytes,
    games: int = DEFAULT_GAMES,
    sims: int = DEFAULT_SIMS,
    mode: str = DEFAULT_MODE,
    game_mode: str = "attrition",
    seed: int | None = None,
) -> dict:
    """
    Generate MCTS self-play dataset on Modal.

    Returns dict with generation stats and output path.
    """
    import subprocess
    import time

    # Extract workspace
    workspace = extract_workspace(workspace_snapshot)

    # Build binary
    if not build_binary(workspace):
        return {"success": False, "error": "Build failed"}

    # Prepare output filename
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    games_str = f"{games // 1000}k" if games >= 1000 else str(games)
    output_filename = f"mcts_{games_str}_sims{sims}_{timestamp}.jsonl.gz"
    output_path = Path(VOLUME_PATH) / output_filename

    # Build command
    cmd = [
        str(workspace / "target" / "release" / "generate_dataset"),
        "--games", str(games),
        "--sims", str(sims),
        "--mode", mode,
        "--game-mode", game_mode,
        "--output", str(output_path),
    ]
    if seed is not None:
        cmd.extend(["--seed", str(seed)])

    print(f"\n[3/4] Generating {games:,} games with {sims} sims...")
    print(f"  Mode: {mode}")
    print(f"  Game mode: {game_mode}")
    print(f"  Output: {output_path}")
    print(f"  Command: {' '.join(cmd)}")
    print()

    gen_start = time.time()

    result = subprocess.run(
        cmd,
        cwd=workspace,
        capture_output=True,
        text=True,
    )

    gen_time = time.time() - gen_start

    print(result.stdout)
    if result.stderr:
        print(f"Stderr: {result.stderr}")

    if result.returncode != 0:
        return {
            "success": False,
            "error": result.stderr,
            "stdout": result.stdout,
        }

    # Get file size
    file_size = output_path.stat().st_size if output_path.exists() else 0

    # Commit volume changes
    print("\n[4/4] Saving to persistent volume...")
    volume.commit()

    return {
        "success": True,
        "output_path": str(output_path),
        "output_filename": output_filename,
        "games": games,
        "sims": sims,
        "generation_time_seconds": gen_time,
        "file_size_bytes": file_size,
        "file_size_mb": file_size / 1024 / 1024,
    }


@app.function(
    volumes={VOLUME_PATH: volume},
    timeout=300,
)
def list_datasets() -> list[dict]:
    """List all datasets in the Modal volume."""
    datasets = []
    volume_path = Path(VOLUME_PATH)

    if not volume_path.exists():
        return datasets

    for f in sorted(volume_path.glob("*.jsonl*")):
        stat = f.stat()
        datasets.append({
            "name": f.name,
            "size_mb": stat.st_size / 1024 / 1024,
            "modified": datetime.fromtimestamp(stat.st_mtime).isoformat(),
        })

    return datasets


@app.function(
    volumes={VOLUME_PATH: volume},
    timeout=1800,  # 30 min for large downloads
)
def download_dataset(filename: str) -> bytes:
    """Download a dataset from the Modal volume."""
    file_path = Path(VOLUME_PATH) / filename

    if not file_path.exists():
        raise FileNotFoundError(f"Dataset not found: {filename}")

    print(f"Reading {filename} ({file_path.stat().st_size / 1024 / 1024:.1f} MB)...")
    return file_path.read_bytes()


# ============================================================================
# Local Entry Point
# ============================================================================

@app.local_entrypoint()
def main(
    games: int = DEFAULT_GAMES,
    sims: int = DEFAULT_SIMS,
    mode: str = DEFAULT_MODE,
    game_mode: str = "attrition",
    seed: int | None = None,
    download: bool = False,
    output: str = "data/datasets",
    list_only: bool = False,
):
    """
    Generate MCTS datasets on Modal cloud.

    Examples:
        modal run modal_dataset.py --games 100000
        modal run modal_dataset.py --games 1000000 --sims 100
        modal run modal_dataset.py --download --output ./datasets
        modal run modal_dataset.py --list-only
    """
    import time

    # List datasets
    if list_only:
        print("\n=== Datasets on Modal Volume ===\n")
        datasets = list_datasets.remote()
        if not datasets:
            print("No datasets found.")
        else:
            total_size = 0
            for d in datasets:
                print(f"  {d['name']:<50} {d['size_mb']:>8.1f} MB  {d['modified']}")
                total_size += d['size_mb']
            print(f"\n  Total: {len(datasets)} datasets, {total_size:.1f} MB")
        return

    # Download mode
    if download:
        print("\n=== Downloading Datasets ===\n")
        datasets = list_datasets.remote()
        if not datasets:
            print("No datasets to download.")
            return

        output_dir = Path(output)
        output_dir.mkdir(parents=True, exist_ok=True)

        for d in datasets:
            local_path = output_dir / d['name']
            if local_path.exists():
                print(f"  Skipping (exists): {d['name']}")
                continue

            print(f"  Downloading: {d['name']} ({d['size_mb']:.1f} MB)...")
            data = download_dataset.remote(d['name'])
            local_path.write_bytes(data)
            print(f"    Saved to: {local_path}")

        print(f"\nDatasets saved to: {output_dir}")
        return

    # Generate mode
    print("\n" + "=" * 60)
    print("Essence Wars Dataset Generation (Modal Cloud)")
    print("=" * 60)
    print(f"  Games:     {games:,}")
    print(f"  MCTS sims: {sims}")
    print(f"  Mode:      {mode}")
    print(f"  Game mode: {game_mode}")
    print(f"  CPUs:      {CPU_COUNT}")
    print("=" * 60)

    # Create workspace snapshot
    print("\nPreparing workspace snapshot...")
    workspace_snapshot = create_workspace_snapshot()

    # Run on Modal
    print("\nStarting Modal job...")
    start_time = time.time()

    result = generate_dataset.remote(
        workspace_snapshot=workspace_snapshot,
        games=games,
        sims=sims,
        mode=mode,
        game_mode=game_mode,
        seed=seed,
    )

    total_time = time.time() - start_time

    # Print results
    print("\n" + "=" * 60)
    print("Generation Complete")
    print("=" * 60)

    if result["success"]:
        print(f"  Games:      {result['games']:,}")
        print(f"  Output:     {result['output_filename']}")
        print(f"  File size:  {result['file_size_mb']:.1f} MB")
        print(f"  Gen time:   {result['generation_time_seconds']:.1f}s")
        print(f"  Total time: {total_time:.1f}s")
        print("\nTo download: modal run modal_dataset.py --download")
    else:
        print(f"  Error: {result.get('error', 'Unknown error')}")
        if result.get('stdout'):
            print(f"\nOutput:\n{result['stdout']}")


if __name__ == "__main__":
    # For local testing, just print help
    print(__doc__)
