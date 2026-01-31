#!/bin/bash
# Batch script to execute all archetype tuning commands
# Auto-generated from tune-commands.txt

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Timestamp function
timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

# Log function
log() {
    echo -e "${BLUE}[$(timestamp)]${NC} $1"
}

success() {
    echo -e "${GREEN}[$(timestamp)]${NC} $1"
}

error() {
    echo -e "${RED}[$(timestamp)]${NC} $1"
}

# Main execution
log "Starting batch archetype tuning..."
log "This will train 4 archetype weight sets (aggro, control, tempo, midrange)"

# Train Aggro weights (5 decks)
log "1/4: Training Aggro weights (5 decks)..."
if cargo run --release --bin tune -- --mode archetype --archetype aggro \
    --generations 100 --games 100 --tag aggro_v1; then
    success "✓ Aggro weights trained successfully"
else
    error "✗ Failed to train Aggro weights"
    exit 1
fi

# Train Control weights (3 decks)
log "2/4: Training Control weights (3 decks)..."
if cargo run --release --bin tune -- --mode archetype --archetype control \
    --generations 100 --games 100 --tag control_v1; then
    success "✓ Control weights trained successfully"
else
    error "✗ Failed to train Control weights"
    exit 1
fi

# Train Tempo weights (3 decks)
log "3/4: Training Tempo weights (3 decks)..."
if cargo run --release --bin tune -- --mode archetype --archetype tempo \
    --generations 100 --games 100 --tag tempo_v1; then
    success "✓ Tempo weights trained successfully"
else
    error "✗ Failed to train Tempo weights"
    exit 1
fi

# Train Midrange weights (1 deck)
log "4/4: Training Midrange weights (1 deck)..."
if cargo run --release --bin tune -- --mode archetype --archetype midrange \
    --generations 100 --games 100 --tag midrange_v1; then
    success "✓ Midrange weights trained successfully"
else
    error "✗ Failed to train Midrange weights"
    exit 1
fi

success "=========================================="
success "All archetype tuning completed successfully!"
success "Weights deployed to: data/weights/archetypes/"
success "=========================================="
