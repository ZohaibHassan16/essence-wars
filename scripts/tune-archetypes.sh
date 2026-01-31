#!/bin/bash
# Tune archetype weights for bot evaluation (used by Greedy, MCTS, and AlphaBeta)
#
# Usage:
#   ./scripts/tune-archetypes.sh              # Run all archetypes
#   ./scripts/tune-archetypes.sh aggro        # Run single archetype
#   ./scripts/tune-archetypes.sh aggro tempo  # Run multiple archetypes
#   ./scripts/tune-archetypes.sh --help       # Show help
#
# Environment variables:
#   GENERATIONS=100  # Number of CMA-ES generations
#   GAMES=100        # Games per evaluation
#   TAG_SUFFIX=v1    # Tag suffix (e.g., aggro_v1)

set -euo pipefail

# Defaults
GENERATIONS="${GENERATIONS:-100}"
GAMES="${GAMES:-100}"
TAG_SUFFIX="${TAG_SUFFIX:-v1}"

# All available archetypes
ALL_ARCHETYPES=(aggro control tempo midrange)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [OPTIONS] [ARCHETYPES...]"
    echo ""
    echo "Tune archetype evaluation weights using CMA-ES optimization."
    echo "These weights are shared by GreedyBot, MctsBot, and AlphaBetaBot."
    echo ""
    echo "Arguments:"
    echo "  ARCHETYPES    Archetypes to tune (default: all)"
    echo "                Available: ${ALL_ARCHETYPES[*]}"
    echo ""
    echo "Options:"
    echo "  -g, --generations N   Number of generations (default: $GENERATIONS)"
    echo "  -n, --games N         Games per evaluation (default: $GAMES)"
    echo "  -t, --tag SUFFIX      Tag suffix (default: $TAG_SUFFIX)"
    echo "  -l, --list            List archetypes and their decks"
    echo "  -d, --dry-run         Show commands without running"
    echo "  -h, --help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                           # Tune all archetypes"
    echo "  $0 aggro                     # Tune only aggro"
    echo "  $0 aggro tempo               # Tune aggro and tempo"
    echo "  $0 -g 50 -n 50 aggro         # Quick test run"
    echo "  GENERATIONS=200 $0 control   # Override via environment"
}

list_archetypes() {
    echo -e "${CYAN}Available archetypes and their decks:${NC}"
    echo ""
    echo -e "${YELLOW}aggro${NC} (5 decks):"
    echo "  - broodmother_pack, alpha_aggro, deathmaster_lethal"
    echo "  - shadow_aggro, artificer_tokens"
    echo ""
    echo -e "${YELLOW}control${NC} (3 decks):"
    echo "  - architect_fortify, healer_sustain, sovereign_control"
    echo ""
    echo -e "${YELLOW}tempo${NC} (3 decks):"
    echo "  - plague_attrition, grove_regenerate, archon_tempo"
    echo ""
    echo -e "${YELLOW}midrange${NC} (1 deck):"
    echo "  - vex_siege"
}

# Parse arguments
DRY_RUN=false
ARCHETYPES=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -g|--generations)
            GENERATIONS="$2"
            shift 2
            ;;
        -n|--games)
            GAMES="$2"
            shift 2
            ;;
        -t|--tag)
            TAG_SUFFIX="$2"
            shift 2
            ;;
        -l|--list)
            list_archetypes
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
        *)
            ARCHETYPES+=("$1")
            shift
            ;;
    esac
done

# Default to all archetypes if none specified
if [[ ${#ARCHETYPES[@]} -eq 0 ]]; then
    ARCHETYPES=("${ALL_ARCHETYPES[@]}")
fi

# Validate archetypes
for arch in "${ARCHETYPES[@]}"; do
    valid=false
    for valid_arch in "${ALL_ARCHETYPES[@]}"; do
        if [[ "$arch" == "$valid_arch" ]]; then
            valid=true
            break
        fi
    done
    if [[ "$valid" == "false" ]]; then
        echo -e "${RED}Invalid archetype: $arch${NC}"
        echo "Available: ${ALL_ARCHETYPES[*]}"
        exit 1
    fi
done

# Header
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Archetype Weight Tuning${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "Archetypes:  ${YELLOW}${ARCHETYPES[*]}${NC}"
echo -e "Generations: ${GENERATIONS}"
echo -e "Games/eval:  ${GAMES}"
echo -e "Tag suffix:  ${TAG_SUFFIX}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN - Commands that would be executed:${NC}"
    echo ""
fi

# Run tuning for each archetype
total=${#ARCHETYPES[@]}
current=0
failed=()

for arch in "${ARCHETYPES[@]}"; do
    current=$((current + 1))
    tag="${arch}_${TAG_SUFFIX}"

    echo -e "${CYAN}[${current}/${total}] Tuning ${YELLOW}${arch}${CYAN} (tag: ${tag})${NC}"

    cmd="cargo run --release --bin tune -- --mode archetype --archetype $arch --generations $GENERATIONS --games $GAMES --tag $tag"

    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  $cmd"
        echo ""
    else
        echo -e "${CYAN}Starting at $(date '+%H:%M:%S')...${NC}"
        echo ""

        if $cmd; then
            echo ""
            echo -e "${GREEN}[${current}/${total}] ${arch} complete!${NC}"
        else
            echo ""
            echo -e "${RED}[${current}/${total}] ${arch} FAILED${NC}"
            failed+=("$arch")
        fi
        echo ""
    fi
done

# Summary
echo -e "${CYAN}========================================${NC}"
if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN complete${NC}"
elif [[ ${#failed[@]} -eq 0 ]]; then
    echo -e "${GREEN}All archetypes tuned successfully!${NC}"
    echo ""
    echo "Weights deployed to: data/weights/archetypes/"
    echo "They will be auto-loaded by arena and validate."
else
    echo -e "${RED}Some archetypes failed: ${failed[*]}${NC}"
    exit 1
fi
echo -e "${CYAN}========================================${NC}"
