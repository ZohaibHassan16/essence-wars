#!/bin/bash
# Symmetric Balance Testing Script
# Runs all faction matchups in both player orders

set -e

GAMES=${1:-100}
echo "=== Symmetric Balance Test (${GAMES} games per matchup) ==="
echo ""

# Define factions and their decks
declare -A FACTIONS=(
    ["argentum"]="argentum_control"
    ["symbiote"]="symbiote_aggro"
    ["obsidion"]="obsidion_burst"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

run_match() {
    local bot1=$1
    local bot2=$2
    local deck1=$3
    local deck2=$4

    cargo run --release --bin arena -- \
        --bot1 "agent-${bot1}" --bot2 "agent-${bot2}" \
        --deck1 "${deck1}" --deck2 "${deck2}" \
        --games "${GAMES}" 2>/dev/null | grep -E "(Bot [12].*wins|Draws)"
}

echo "=============================================="
echo "ARGENTUM vs SYMBIOTE"
echo "=============================================="
echo "--- Argentum P1 ---"
run_match argentum symbiote argentum_control symbiote_aggro
echo ""
echo "--- Symbiote P1 ---"
run_match symbiote argentum symbiote_aggro argentum_control
echo ""

echo "=============================================="
echo "ARGENTUM vs OBSIDION"
echo "=============================================="
echo "--- Argentum P1 ---"
run_match argentum obsidion argentum_control obsidion_burst
echo ""
echo "--- Obsidion P1 ---"
run_match obsidion argentum obsidion_burst argentum_control
echo ""

echo "=============================================="
echo "SYMBIOTE vs OBSIDION"
echo "=============================================="
echo "--- Symbiote P1 ---"
run_match symbiote obsidion symbiote_aggro obsidion_burst
echo ""
echo "--- Obsidion P1 ---"
run_match obsidion symbiote obsidion_burst symbiote_aggro
echo ""

echo "=============================================="
echo "Test Complete!"
echo "=============================================="
