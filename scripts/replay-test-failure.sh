#!/bin/bash
# Replay a failed test from experiments/test_failures/
#
# Usage: ./scripts/replay-test-failure.sh <failure_file>
# Example: ./scripts/replay-test-failure.sh experiments/test_failures/1737059234_seed_12345.txt

set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <failure_file>"
    echo ""
    echo "Available failures:"
    ls -1t experiments/test_failures/*.txt 2>/dev/null | head -5 || echo "  (none found)"
    exit 1
fi

FAILURE_FILE="$1"

if [ ! -f "$FAILURE_FILE" ]; then
    echo "Error: File not found: $FAILURE_FILE"
    exit 1
fi

echo "=== Replaying Test Failure ==="
echo ""
cat "$FAILURE_FILE"
echo ""

# Extract seed from filename or file content
SEED=$(grep "^Seed:" "$FAILURE_FILE" | cut -d' ' -f2)
DECK1=$(grep "^Deck 1:" "$FAILURE_FILE" | cut -d' ' -f3-)
DECK2=$(grep "^Deck 2:" "$FAILURE_FILE" | cut -d' ' -f3-)
BOT1=$(grep "^Bot 1:" "$FAILURE_FILE" | cut -d' ' -f3-)
BOT2=$(grep "^Bot 2:" "$FAILURE_FILE" | cut -d' ' -f3-)

echo "Extracted configuration:"
echo "  Seed: $SEED"
echo "  Deck1: $DECK1"
echo "  Deck2: $DECK2"
echo "  Bot1: $BOT1"
echo "  Bot2: $BOT2"
echo ""

# Create a temporary test file to reproduce the exact scenario
cat > /tmp/replay_test.rs <<EOF
#[test]
fn replay_failure_seed_${SEED}() {
    use cardgame::cards::CardDatabase;
    use cardgame::decks::DeckRegistry;
    use cardgame::engine::GameEngine;
    use cardgame::types::CardId;
    
    let card_db = CardDatabase::load_from_directory("data/cards/core_set")
        .expect("Failed to load cards");
    let deck_registry = DeckRegistry::load_from_directory("data/decks")
        .expect("Failed to load decks");
    
    let deck1 = deck_registry.get("${DECK1}").expect("Deck 1 not found");
    let deck2 = deck_registry.get("${DECK2}").expect("Deck 2 not found");
    
    let deck1_cards: Vec<CardId> = deck1.cards.iter().map(|&id| CardId(id)).collect();
    let deck2_cards: Vec<CardId> = deck2.cards.iter().map(|&id| CardId(id)).collect();
    
    let engine = GameEngine::new(&card_db, deck1_cards, deck2_cards, ${SEED});
    
    // Run game to reproduce bug
    println!("Replaying game with seed ${SEED}...");
    // Add replay logic here
}
EOF

echo "📝 Temporary replay test created at /tmp/replay_test.rs"
echo ""
echo "To reproduce manually:"
echo "  1. Add the test to tests/regression/ directory"
echo "  2. Run: cargo test replay_failure_seed_${SEED} -- --nocapture"
echo ""
echo "Or run arena with these settings:"
echo "  cargo run --release --bin arena -- \\"
echo "    --deck1 $DECK1 --deck2 $DECK2 \\"
echo "    --bot1 $(echo $BOT1 | tr '[:upper:]' '[:lower:]') \\"
echo "    --bot2 $(echo $BOT2 | tr '[:upper:]' '[:lower:]') \\"
echo "    --games 1 --seed $SEED --debug"
