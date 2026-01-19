#!/bin/bash
# Train the full PPO Agent Roster for Essence Wars
#
# This script trains 7 PPO agents with improvements to reduce policy collapse:
#   - Higher entropy coefficient (0.02)
#   - Best checkpoint saving (automatically saves best model)
#   - Embedded architecture for faction specialists
#
# Agents:
#   - 4 generalist variants (architecture comparison)
#   - 3 faction specialists (using embedded architecture)
#
# Estimated time: ~30-45 min total
#
# Usage:
#   ./scripts/train_ppo_roster.sh           # Run all 7 agents
#   ./scripts/train_ppo_roster.sh --quick   # Quick test (10k steps each)

set -e  # Exit on error

# Configuration
TIMESTEPS=300000
EVAL_INTERVAL=25000
ENT_COEF=0.02
CARD2VEC_PATH="models/card2vec_20260119_120507.pt"

# Check for quick mode
if [[ "$1" == "--quick" ]]; then
    echo "Running in QUICK TEST mode (10k steps per agent)"
    TIMESTEPS=10000
    EVAL_INTERVAL=5000
fi

echo "============================================================"
echo "PPO Agent Roster Training (v2 - with improvements)"
echo "============================================================"
echo "  Timesteps per agent: $TIMESTEPS"
echo "  Entropy coefficient: $ENT_COEF (higher to reduce collapse)"
echo "  Best checkpoint:     enabled (saves best_model.pt)"
echo "  Total agents: 7"
echo "  Card2Vec path: $CARD2VEC_PATH"
echo "============================================================"
echo ""

# Check Card2Vec exists
if [[ ! -f "$CARD2VEC_PATH" ]]; then
    echo "Warning: Card2Vec embeddings not found at $CARD2VEC_PATH"
    echo "Pretrained embedding agents will be skipped."
    SKIP_PRETRAINED=1
fi

# Track results
RESULTS_FILE="experiments/ppo/roster_results_$(date +%Y%m%d_%H%M%S).txt"
mkdir -p experiments/ppo
echo "PPO Roster Training Results v2 - $(date)" > "$RESULTS_FILE"
echo "==========================================" >> "$RESULTS_FILE"
echo "Improvements: ent_coef=$ENT_COEF, best checkpoint, embedded specialists" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

train_agent() {
    local name="$1"
    local args="$2"

    echo ""
    echo "------------------------------------------------------------"
    echo "Training: $name"
    echo "------------------------------------------------------------"

    START_TIME=$(date +%s)

    uv run python python/scripts/train_ppo.py \
        --timesteps $TIMESTEPS \
        --eval-interval $EVAL_INTERVAL \
        --ent-coef $ENT_COEF \
        $args

    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    echo "$name: completed in ${DURATION}s" >> "$RESULTS_FILE"
    echo "Completed: $name (${DURATION}s)"
}

# ============================================================
# Tier 1: Architecture Comparison (Generalists)
# ============================================================

echo ""
echo "============================================================"
echo "TIER 1: Architecture Comparison (4 Generalists)"
echo "============================================================"

# 1. Learned embeddings (best performer from previous run)
train_agent "ppo-generalist-embedded" "--observation-mode embedded"

# 2. Flat baseline
train_agent "ppo-generalist-flat" "--observation-mode flat"

# 3. Pretrained Card2Vec (fine-tuned)
if [[ -z "$SKIP_PRETRAINED" ]]; then
    train_agent "ppo-generalist-pretrained" \
        "--observation-mode embedded_pretrained --pretrained-embeds $CARD2VEC_PATH"

    # 4. Pretrained Card2Vec (frozen)
    train_agent "ppo-generalist-pretrained-frozen" \
        "--observation-mode embedded_pretrained --pretrained-embeds $CARD2VEC_PATH --freeze-embeds"
else
    echo "Skipping pretrained embedding agents (Card2Vec not found)"
fi

# ============================================================
# Tier 2: Faction Specialists (Using embedded architecture)
# ============================================================

echo ""
echo "============================================================"
echo "TIER 2: Faction Specialists (3 agents, embedded architecture)"
echo "============================================================"

# 5. Argentum specialist (embedded to reduce collapse)
train_agent "ppo-argentum-specialist" "--player-faction argentum --observation-mode embedded"

# 6. Symbiote specialist (embedded to reduce collapse)
train_agent "ppo-symbiote-specialist" "--player-faction symbiote --observation-mode embedded"

# 7. Obsidion specialist (embedded to reduce collapse)
train_agent "ppo-obsidion-specialist" "--player-faction obsidion --observation-mode embedded"

# ============================================================
# Summary
# ============================================================

echo ""
echo "============================================================"
echo "TRAINING COMPLETE"
echo "============================================================"
echo ""
echo "Results saved to: $RESULTS_FILE"
cat "$RESULTS_FILE"
echo ""
echo "Models saved to: experiments/ppo/"
ls -la experiments/ppo/ | grep "$(date +%Y%m%d)" | head -10
echo ""
echo "Note: Each agent folder contains:"
echo "  - final_model.pt (last checkpoint)"
echo "  - best_model.pt (highest eval win rate)"
echo "  - summary.txt (training stats)"
echo ""
echo "Next steps:"
echo "  1. Compare best_model.pt vs final_model.pt for each agent"
echo "  2. Upload to HuggingFace: uv run python python/scripts/upload_models.py"
