# Shell Completions for essence-wars CLI

This directory contains shell completion scripts for the `essence-wars` command-line tool.

## Installation

### Bash

Add to your `~/.bashrc`:

```bash
# Option 1: Source directly (slower startup but always up-to-date)
eval "$(_ESSENCE_WARS_COMPLETE=bash_source essence-wars)"

# Option 2: Use pre-generated script (faster startup)
source /path/to/essence-wars/python/completions/essence-wars.bash
```

### Zsh

Add to your `~/.zshrc`:

```zsh
# Option 1: Source directly
eval "$(_ESSENCE_WARS_COMPLETE=zsh_source essence-wars)"

# Option 2: Use pre-generated script
source /path/to/essence-wars/python/completions/essence-wars.zsh
```

### Fish

Copy to Fish completions directory:

```fish
# Option 1: Source directly (add to config.fish)
_ESSENCE_WARS_COMPLETE=fish_source essence-wars | source

# Option 2: Copy pre-generated script
cp /path/to/essence-wars/python/completions/essence-wars.fish ~/.config/fish/completions/
```

## Regenerating Completions

If the CLI changes, regenerate the completion scripts:

```bash
# Bash
_ESSENCE_WARS_COMPLETE=bash_source essence-wars > completions/essence-wars.bash

# Zsh
_ESSENCE_WARS_COMPLETE=zsh_source essence-wars > completions/essence-wars.zsh

# Fish
_ESSENCE_WARS_COMPLETE=fish_source essence-wars > completions/essence-wars.fish
```

## Usage

After installation, you can use tab completion with the `essence-wars` command:

```bash
essence-wars <TAB>
# Shows: benchmark  data  evaluate-mcts  report  train  validate

essence-wars train <TAB>
# Shows: ppo  alphazero  behavioral-cloning  card2vec  ...

essence-wars train ppo --<TAB>
# Shows: --timesteps  --num-envs  --lr  --help  ...
```
