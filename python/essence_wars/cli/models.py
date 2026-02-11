"""Model management commands for Essence Wars."""

from __future__ import annotations

import subprocess
import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import click


def create_models_group() -> click.Group:
    """Create the models command group."""
    import click

    @click.group()
    def models() -> None:
        """Manage neural agent models.

        \b
        Commands:
          status    - Show status of registered models
          train     - Train models (missing or specific)
          validate  - Validate a model can be loaded
          list      - List all discovered models
        """

    @models.command("status")
    def models_status() -> None:
        """Show status of all registered models.

        \b
        Examples:
          essence-wars models status
        """
        from essence_wars.models import ModelRegistry

        try:
            registry = ModelRegistry.load()
        except FileNotFoundError as e:
            print(f"Error: {e}")
            sys.exit(1)

        print(registry.status())

    @models.command("train")
    @click.argument("model_names", nargs=-1)
    @click.option("--missing", is_flag=True, help="Train all missing models")
    @click.option("--all", "train_all", is_flag=True, help="Train all models")
    @click.option("--dry-run", is_flag=True, help="Show commands without running")
    def models_train(
        model_names: tuple[str, ...],
        missing: bool,
        train_all: bool,
        dry_run: bool,
    ) -> None:
        """Train neural agent models.

        \b
        Examples:
          essence-wars models train --missing
          essence-wars models train ppo-aggro ppo-control
          essence-wars models train --all --dry-run
        """
        from essence_wars.models import ModelRegistry

        try:
            registry = ModelRegistry.load()
        except FileNotFoundError as e:
            print(f"Error: {e}")
            sys.exit(1)

        # Determine which models to train
        if missing:
            to_train = registry.missing
            if not to_train:
                print("All models are already available!")
                return
        elif train_all:
            to_train = list(registry.models)
        elif model_names:
            to_train = []
            for name in model_names:
                model = registry.get(name)
                if model is None:
                    print(f"Error: Unknown model '{name}'")
                    print(f"Available: {', '.join(m.name for m in registry.models)}")
                    sys.exit(1)
                to_train.append(model)
        else:
            print("Specify models to train, --missing, or --all")
            sys.exit(1)

        if not to_train:
            print("No models to train.")
            return

        print(f"Training {len(to_train)} model(s)...\n")

        failed = []
        for i, model in enumerate(to_train, 1):
            print(f"[{i}/{len(to_train)}] Training {model.name}...")
            print(f"  Command: {model.train_cmd}")

            if dry_run:
                print("  (dry run - skipping)")
                continue

            result = subprocess.run(model.train_cmd, shell=True, check=False)
            if result.returncode == 0:
                print("  Complete")
            else:
                print(f"  Failed (exit code {result.returncode})")
                failed.append(model.name)
            print()

        if failed:
            print(f"Failed: {', '.join(failed)}")
            sys.exit(1)
        elif not dry_run:
            print("All models trained successfully!")

    @models.command("validate")
    @click.argument("model_name")
    def models_validate(model_name: str) -> None:
        """Validate that a model can be loaded.

        \b
        Examples:
          essence-wars models validate ppo-generalist
        """
        from essence_wars.models import ModelRegistry

        try:
            registry = ModelRegistry.load()
        except FileNotFoundError as e:
            print(f"Error: {e}")
            sys.exit(1)

        model = registry.get(model_name)
        if model is None:
            print(f"Error: Unknown model '{model_name}'")
            sys.exit(1)

        if not model.is_available:
            print(f"Error: Model '{model_name}' not found")
            print(f"Train it with: {model.train_cmd}")
            sys.exit(1)

        print(f"Validating {model.name}...")
        print(f"  Path: {model.discovered_path}")

        try:
            agent = model.load_agent(device="cpu")
            print(f"  Loaded successfully as '{agent.name}'")
        except Exception as e:
            print(f"  Failed to load: {e}")
            sys.exit(1)

    @models.command("list")
    def models_list() -> None:
        """List all discovered models (not just registered).

        \b
        Examples:
          essence-wars models list
        """
        from essence_wars.models.discovery import discover_all_models, get_model_info

        models_found = discover_all_models()

        if not models_found:
            print("No models found in python/experiments/")
            return

        total = sum(len(v) for v in models_found.values())
        print(f"Found {total} model(s) in python/experiments/\n")

        for model_type, checkpoints in sorted(models_found.items()):
            print(f"{model_type.upper()}:")
            for checkpoint in checkpoints[:5]:
                try:
                    info = get_model_info(checkpoint)
                    size = f"{info['size_mb']:.1f}MB"
                    print(
                        f"  {info['name']:<40} {size:>8}  {info['modified'][:10]}"
                    )
                except Exception:
                    print(f"  {checkpoint.parent.name}")

            if len(checkpoints) > 5:
                print(f"  ... and {len(checkpoints) - 5} more")
            print()

    return models
