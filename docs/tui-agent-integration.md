# TUI Agent Integration TODO

**Status**: Pending
**Priority**: Medium
**Related**: Agent Architecture (see `docs/essence-wars-design.md` Section 19)

---

## Overview

The TUI currently supports basic bot selection (Random, Greedy, MCTS). It needs updates to support the new Agent-type system with faction specialists and generalist bots.

---

## Required Changes

### 1. Bot Selection Screen

**File**: `src/tui/screens/bot_select.rs` (or equivalent)

Add new bot options:
- `Agent-Argentum` - Argentum Combine specialist
- `Agent-Symbiote` - Symbiote Circles specialist
- `Agent-Obsidion` - Obsidion Syndicate specialist
- `Agent-Generalist` - Works with all factions

**UI Considerations**:
- Group Agent types separately from basic bots (Random/Greedy/MCTS)
- Show faction icon/color for specialists
- Indicate if specialist weights are loaded or using defaults

### 2. Deck Selection Integration

**Validation Logic**:
When an Agent specialist is selected, the deck selection should:
1. Filter to show only compatible faction decks (recommended)
2. Or show warning icon on incompatible decks
3. Display faction tag next to deck names

**Example Flow**:
```
Player selects: Agent-Argentum
Deck list shows:
  [Argentum] argentum_control  ✓ Compatible
  [Symbiote] symbiote_aggro    ⚠ Warning: Different faction
```

### 3. Weight Loading Indicator

Show weight loading status in bot info panel:
- "Weights: agent_argentum (loaded)" - specialist weights found
- "Weights: default (no specialist file)" - fallback to defaults

**File paths to check**:
- `data/weights/specialists/{faction}.toml`
- `data/weights/generalist.toml`

### 4. Arena Screen Updates

**File**: `src/tui/screens/arena.rs`

Update to handle Agent bot types:
- Pass correct `BotType` enum variants
- Auto-load weights using `BotType::agent_weights_path()`
- Show faction-deck binding warnings in match setup

### 5. Tuning Screen Updates

**File**: `src/tui/screens/tuning.rs`

Add new tuning mode options:
- `faction-specialist` with faction selector dropdown
- `agent-generalist` mode

UI elements needed:
- Faction selector (Argentum/Symbiote/Obsidion)
- Display target weight file path
- Show training progress with faction context

---

## Implementation Notes

### Dependencies

The core logic is already in place:
- `Faction` enum in `src/decks.rs`
- `BotType::AgentSpecialist(Faction)` in arena.rs
- Weight auto-loading via `agent_weights_path()`
- Faction-deck validation via `validate_faction_deck_binding()`

### Code Patterns to Follow

```rust
// Faction display
use cardgame::decks::Faction;

let faction = Faction::Argentum;
let display = faction.display_name();  // "Argentum Combine"
let tag = faction.as_tag();            // "argentum"

// Deck faction detection
let deck = registry.get("argentum_control").unwrap();
if let Some(faction) = deck.faction() {
    // deck belongs to a faction
}

// Get compatible decks for a specialist
let decks = registry.decks_for_specialist(Faction::Argentum);
```

### Suggested UI Components

1. **FactionBadge** - Small colored label showing faction
2. **BotTypeSelector** - Radio group with Agent types section
3. **WeightStatusIndicator** - Shows loaded/default status

---

## Testing Checklist

- [ ] Agent bot types appear in bot selection
- [ ] Deck filtering works for specialists
- [ ] Faction-deck warnings display correctly
- [ ] Weight loading status is accurate
- [ ] Tuning screen shows faction-specialist option
- [ ] Arena matches work with Agent types

---

## Related Files

- `src/tui/screens/` - Screen implementations
- `src/tui/components/` - Reusable UI components
- `src/bin/arena.rs` - Reference implementation for Agent handling
- `src/decks.rs` - Faction enum and deck methods
