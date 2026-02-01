# Design: Human Player Ability Activation in Tauri UI

## Overview

This document designs the UI flow for human players to activate creature abilities (including tokens) in the Tauri desktop app. The game engine already fully supports abilities - this is purely a frontend concern.

## Current State

### Backend (Fully Implemented)
- **TokenAbility struct**: `name`, `essence_cost`, `targeting: TargetingRule`, `effects: Vec<TokenEffect>`
- **TargetingRule enum**: `NoTarget`, `TargetEnemyCreature`, `TargetEnemyPlayer`, `TargetAny`, `TargetAllyCreature`, etc.
- **Legal action generation**: Filters by essence cost, silenced state, stealth visibility
- **Action execution**: Deducts essence, applies effects, handles self-sacrifice
- **ActionInfo DTO**: Already handles `use_ability` with `source_slot` and `target_slot`

### Frontend (NOT Implemented)
- `CreatureDto` has NO ability information
- `GameStore` only handles attack targeting, ignores ability actions
- No visual indicator for creatures with abilities
- No ability menu/selection UI
- No targeting flow for abilities

## Design Goals

1. **Unified creature interaction**: Clicking a creature shows all available actions (attack + abilities)
2. **Clear affordances**: Visual indicators for creatures with usable abilities
3. **Consistent targeting**: Reuse existing attack targeting flow where possible
4. **Minimal complexity**: Simple menu, no nested modals

## Data Model Changes

### 1. Add AbilityDto to types.ts

```typescript
export interface AbilityDto {
  index: number;           // 0-5, matches ability_index in Action::UseAbility
  name: string;            // Display name (e.g., "Fungal Rot")
  essenceCost: number;     // Cost to activate
  targetingType: string;   // "no_target" | "enemy_creature" | "enemy_player" | "any" | etc.
  description: string;     // Human-readable effect description
  isUsable: boolean;       // Can the player afford it and is there a valid target?
}

export interface CreatureDto {
  // ... existing fields ...
  abilities: AbilityDto[];  // NEW: List of activated abilities (empty for most creatures)
}
```

### 2. Update ActionInfo for Ability Context

```typescript
export interface ActionInfo {
  // ... existing fields ...
  abilityIndex?: number;   // NEW: Which ability (0-5) for use_ability actions
  abilityName?: string;    // NEW: Ability name for display
}
```

### 3. Backend Serialization Changes

Update `CreatureDto::from_creature()` and `CreatureDto::from_token()` in `serialization.rs`:

```rust
impl CreatureDto {
    pub fn from_creature(creature: &Creature, card: &CardDefinition, current_turn: u16, player_essence: u8) -> Self {
        // ... existing code ...

        let abilities = creature.token_abilities
            .as_ref()
            .map(|abs| abs.iter().enumerate().map(|(i, a)| {
                AbilityDto {
                    index: i as u8,
                    name: a.name.clone(),
                    essence_cost: a.essence_cost,
                    targeting_type: targeting_rule_to_string(&a.targeting),
                    description: ability_description(&a.effects),
                    is_usable: player_essence >= a.essence_cost, // Simplified check
                }
            }).collect())
            .unwrap_or_default();

        Self {
            // ... existing fields ...
            abilities,
        }
    }
}
```

## UI Component Changes

### 1. CreatureSlot.svelte - Add Ability Indicator

Show a visual indicator when a creature has usable abilities:

```svelte
<!-- Ability indicator (spark/glow icon) -->
{#if creature.abilities.length > 0 && creature.abilities.some(a => a.isUsable)}
  <div class="absolute top-2 left-2 w-4 h-4 rounded-full bg-purple-500 animate-pulse
              flex items-center justify-center">
    <span class="text-xs">✦</span>
  </div>
{/if}
```

### 2. New Component: CreatureActionMenu.svelte

A floating menu that appears when clicking a creature with multiple action options:

```svelte
<script lang="ts">
  import type { CreatureDto, AbilityDto, ActionInfo } from "$lib/api/types";

  let {
    creature,
    canAttack,
    legalAbilityActions,  // Filtered from legalActions
    onAttack,
    onAbility,
    onClose,
    position,
  }: {
    creature: CreatureDto;
    canAttack: boolean;
    legalAbilityActions: ActionInfo[];
    onAttack: () => void;
    onAbility: (abilityIndex: number) => void;
    onClose: () => void;
    position: { x: number; y: number };
  } = $props();

  // Group legal ability actions by ability index
  const usableAbilities = $derived(() => {
    const indices = new Set(
      legalAbilityActions.map(a => a.abilityIndex).filter(i => i !== undefined)
    );
    return creature.abilities.filter(a => indices.has(a.index));
  });
</script>

<div
  class="absolute z-50 bg-gray-900 border border-gray-600 rounded-lg shadow-xl p-2 min-w-[160px]"
  style="left: {position.x}px; top: {position.y}px;"
>
  <!-- Attack option -->
  {#if canAttack}
    <button
      class="w-full text-left px-3 py-2 rounded hover:bg-gray-700 flex items-center gap-2"
      onclick={onAttack}
    >
      <span class="text-red-400">⚔</span>
      <span>Attack</span>
    </button>
  {/if}

  <!-- Ability options -->
  {#each usableAbilities as ability}
    <button
      class="w-full text-left px-3 py-2 rounded hover:bg-gray-700 flex items-center gap-2"
      onclick={() => onAbility(ability.index)}
    >
      <span class="text-purple-400">✦</span>
      <div class="flex-1">
        <div class="text-sm">{ability.name}</div>
        <div class="text-xs text-gray-400">{ability.description}</div>
      </div>
      {#if ability.essenceCost > 0}
        <span class="text-blue-400 text-sm">{ability.essenceCost}⬡</span>
      {/if}
    </button>
  {/each}

  <!-- Close button if menu was opened but no actions available -->
  {#if !canAttack && usableAbilities.length === 0}
    <div class="px-3 py-2 text-gray-500 text-sm">No actions available</div>
  {/if}
</div>
```

### 3. GameStore State Changes

Add ability selection state to `gameState.svelte.ts`:

```typescript
class GameStore {
  // ... existing state ...

  // Ability selection state
  selectedAbilityIndex = $state<number | null>(null);
  actionMenuPosition = $state<{ x: number; y: number } | null>(null);

  // Computed: Whether to show action menu vs direct targeting
  get showActionMenu() {
    if (this.selectedCreatureSlot === null) return false;
    const creature = this.gameState?.player.creatures[this.selectedCreatureSlot];
    if (!creature) return false;

    // Show menu if creature can both attack AND has abilities
    const canAttack = this.hasAttackActions(this.selectedCreatureSlot);
    const hasAbilities = this.hasAbilityActions(this.selectedCreatureSlot);
    return canAttack && hasAbilities;
  }

  hasAttackActions(slot: number): boolean {
    return this.legalActions.some(
      a => a.actionType === "attack" && a.sourceSlot === slot
    );
  }

  hasAbilityActions(slot: number): boolean {
    return this.legalActions.some(
      a => a.actionType === "use_ability" && a.sourceSlot === slot
    );
  }

  getAbilityActions(slot: number, abilityIndex?: number): ActionInfo[] {
    return this.legalActions.filter(a =>
      a.actionType === "use_ability" &&
      a.sourceSlot === slot &&
      (abilityIndex === undefined || a.abilityIndex === abilityIndex)
    );
  }

  selectCreature(slot: number, event?: MouseEvent) {
    if (this.selectedCreatureSlot === slot && this.selectedAbilityIndex === null) {
      this.clearSelection();
      return;
    }

    this.selectedCreatureSlot = slot;
    this.selectedCardIndex = null;
    this.selectedAbilityIndex = null;

    // Determine if we need action menu or can go straight to targeting
    const canAttack = this.hasAttackActions(slot);
    const hasAbilities = this.hasAbilityActions(slot);

    if (canAttack && hasAbilities && event) {
      // Show action menu at click position
      this.actionMenuPosition = { x: event.clientX, y: event.clientY };
    } else if (canAttack) {
      // Direct attack targeting (existing behavior)
      this.updateHighlights();
    } else if (hasAbilities) {
      // Only abilities - check if single ability with NoTarget
      const abilities = this.getAbilityActions(slot);
      const creature = this.gameState?.player.creatures[slot];

      if (creature?.abilities.length === 1) {
        const ability = creature.abilities[0];
        if (ability.targetingType === "no_target" || ability.targetingType === "enemy_player") {
          // Auto-execute no-target ability
          this.executeAbility(0);
        } else {
          // Single targeted ability - go to targeting mode
          this.selectAbility(0);
        }
      } else if (event) {
        // Multiple abilities - show menu
        this.actionMenuPosition = { x: event.clientX, y: event.clientY };
      }
    }
  }

  selectAbility(abilityIndex: number) {
    this.selectedAbilityIndex = abilityIndex;
    this.actionMenuPosition = null;
    this.updateHighlights();
  }

  selectAttackMode() {
    this.selectedAbilityIndex = null;
    this.actionMenuPosition = null;
    this.updateHighlights();
  }

  updateHighlights() {
    const slots: number[] = [];

    if (this.selectedCardIndex !== null) {
      // Card play targeting (existing)
      for (const action of this.legalActions) {
        if (action.actionType === "play_card" && action.handIndex === this.selectedCardIndex) {
          if (action.targetSlot !== undefined) {
            slots.push(action.targetSlot);
          }
        }
      }
    } else if (this.selectedCreatureSlot !== null) {
      if (this.selectedAbilityIndex !== null) {
        // Ability targeting - highlight valid targets from legal actions
        for (const action of this.legalActions) {
          if (action.actionType === "use_ability" &&
              action.sourceSlot === this.selectedCreatureSlot &&
              action.abilityIndex === this.selectedAbilityIndex) {
            if (action.targetSlot !== undefined) {
              slots.push(action.targetSlot);
            }
            // Note: action.targetSlot === undefined means NoTarget/face
          }
        }
      } else {
        // Attack targeting (existing)
        for (const action of this.legalActions) {
          if (action.actionType === "attack" && action.sourceSlot === this.selectedCreatureSlot) {
            if (action.targetSlot !== undefined) {
              slots.push(action.targetSlot);
            }
          }
        }
      }
    }

    this.highlightedSlots = slots;
  }

  async executeAbility(abilityIndex: number) {
    if (this.selectedCreatureSlot === null) return;

    // Find the action for this ability (no-target abilities)
    const action = this.legalActions.find(a =>
      a.actionType === "use_ability" &&
      a.sourceSlot === this.selectedCreatureSlot &&
      a.abilityIndex === abilityIndex &&
      a.targetSlot === undefined
    );

    if (action) {
      await this.applyAction(action.index);
    }
  }

  getActionForTarget(targetSlot: number): ActionInfo | null {
    if (this.selectedCardIndex !== null) {
      return this.legalActions.find(
        a => a.actionType === "play_card" &&
             a.handIndex === this.selectedCardIndex &&
             a.targetSlot === targetSlot
      ) ?? null;
    } else if (this.selectedCreatureSlot !== null) {
      if (this.selectedAbilityIndex !== null) {
        // Find ability action for this target
        return this.legalActions.find(
          a => a.actionType === "use_ability" &&
               a.sourceSlot === this.selectedCreatureSlot &&
               a.abilityIndex === this.selectedAbilityIndex &&
               a.targetSlot === targetSlot
        ) ?? null;
      } else {
        // Find attack action (existing)
        return this.legalActions.find(
          a => a.actionType === "attack" &&
               a.sourceSlot === this.selectedCreatureSlot &&
               a.targetSlot === targetSlot
        ) ?? null;
      }
    }
    return null;
  }

  clearSelection() {
    this.selectedCardIndex = null;
    this.selectedCreatureSlot = null;
    this.selectedAbilityIndex = null;
    this.actionMenuPosition = null;
    this.highlightedSlots = [];
  }
}
```

## Interaction Flow

### Scenario 1: Creature with Only Attack

1. Player clicks own creature
2. `selectCreature(slot)` called
3. No abilities → direct attack targeting mode
4. Valid attack targets highlighted
5. Player clicks target → attack executed

### Scenario 2: Creature with Only Abilities (e.g., Token)

1. Player clicks own token
2. `selectCreature(slot)` called
3. No attack, has abilities → check ability count

**If single no-target ability:**
- Execute immediately (e.g., Soul Siphon targeting face)

**If single targeted ability:**
- Enter targeting mode for that ability
- Valid targets highlighted
- Player clicks target → ability executed

**If multiple abilities:**
- Show action menu at cursor position
- Player selects ability
- If no-target: execute immediately
- If targeted: enter targeting mode

### Scenario 3: Creature with Attack AND Abilities

1. Player clicks own creature
2. `selectCreature(slot)` called
3. Has attack AND abilities → show action menu
4. Menu displays: "Attack" + each ability
5. Player selects action
6. If Attack: enter attack targeting mode
7. If Ability: depends on targeting type

### Scenario 4: Face-Targeting Abilities

For abilities with `TargetEnemyPlayer` or `TargetAny` (when targeting face):
- The action has `targetSlot === undefined`
- UI needs a way to select "enemy commander" as target
- Options:
  a. Click on opponent commander portrait
  b. Highlighted "FACE" indicator appears
  c. Menu option "Target Enemy Commander"

**Recommended: Commander click target**
- When ability requires/allows face targeting, highlight opponent commander
- Clicking opponent commander executes the ability
- Consistent with attack-face actions (if implemented)

## Visual Design

### Ability Indicator on Creature Card
```
┌──────────────────┐
│ ✦              ⚡ │  ← ✦ = has usable abilities, ⚡ = can attack
│   [CREATURE ART]  │
│                   │
│   Creature Name   │
│   ⚔ 3   ♥ 4      │
└──────────────────┘
```

### Action Menu
```
┌─────────────────────┐
│  ⚔ Attack           │
├─────────────────────┤
│  ✦ Fungal Rot       │
│    -1/-1 to enemy   │
│              1⬡     │
├─────────────────────┤
│  ✦ Volatile Burst   │
│    2 damage to any  │
│              0⬡     │
└─────────────────────┘
```

### Targeting Mode Indicator
When in ability targeting mode, show the ability name:
```
┌─────────────────────────────────────┐
│  Using: Fungal Rot  [Cancel]        │
└─────────────────────────────────────┘
```

## Implementation Order

### Phase 1: Data Layer
1. Add `AbilityDto` to TypeScript types
2. Add `abilities` field to `CreatureDto` in Rust serialization
3. Add `abilityIndex` and `abilityName` to `ActionInfo`
4. Update `CreatureDto::from_token()` to populate abilities

### Phase 2: State Management
1. Add `selectedAbilityIndex` and `actionMenuPosition` to GameStore
2. Update `selectCreature()` to handle ability-equipped creatures
3. Update `updateHighlights()` to handle ability targeting
4. Update `getActionForTarget()` to find ability actions

### Phase 3: UI Components
1. Add ability indicator to `CreatureSlot.svelte`
2. Create `CreatureActionMenu.svelte` component
3. Update `GameBoard.svelte` to render action menu
4. Add targeting mode indicator bar

### Phase 4: Face Targeting
1. Make opponent commander clickable during ability targeting
2. Add `canTargetFace` derived state
3. Handle face-targeting actions

### Phase 5: Polish
1. Sound effects for ability activation
2. Animation for ability effects
3. Keyboard shortcuts (1-6 for abilities?)
4. Tooltip improvements

## Testing Checklist

- [ ] Token with single no-target ability (Soul Siphon) executes on click
- [ ] Token with single targeted ability (Fungal Rot) enters targeting mode
- [ ] Creature with attack + ability shows action menu
- [ ] Ability targeting respects Stealth (stealthed creatures not targetable)
- [ ] Essence cost checked (can't use ability you can't afford)
- [ ] Face-targeting abilities can target commander
- [ ] Silenced creatures can't use abilities (no ability indicator shown)
- [ ] AI hint shows ability recommendations correctly
- [ ] Spectator mode displays ability actions in action log

## Open Questions

1. **Should abilities cost Action Points?** Currently they don't - only essence.
2. **Animation for self-sacrifice?** Creature dies after ability - need death animation.
3. **Keyboard shortcuts?** Number keys for quick ability access?
4. **Undo support?** Should ability actions be undoable?

## Appendix: Existing Commander Token Abilities

| Commander | Token | Ability | Targeting | Effect |
|-----------|-------|---------|-----------|--------|
| High Artificer | Brass Cog | Volatile Overload | TargetAny | 2 damage, destroy self |
| Plague Sovereign | Sporeling | Fungal Rot | TargetEnemyCreature | -1/-1, destroy self |
| Shadow Weaver | Shadow Wisp | Soul Siphon | NoTarget (face) | 1 damage to enemy, heal 2, destroy self |

All current token abilities include `DestroySelf` - they're one-time use sacrificial effects.
