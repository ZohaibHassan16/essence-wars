# Phase 5A-3: Visual Foundation - Gap Analysis & Implementation Tracker

> **Created**: 2026-01-20
> **Status**: Tier 1 & Tier 2 Complete
> **Overall Completion**: 100% (All planned items implemented)

---

## Executive Summary

Phase 5A-3 aims to establish the "Farsight Table" aesthetic - a war room command table floating in a dark void with faction-themed materials, crystal gem tokens, and atmospheric lighting. The current implementation has solid foundations but lacks critical atmospheric and polish elements.

---

## Gap Inventory

### Tier 1: Essential for Phase 5A-3 Completion

| ID | Feature | Current State | Gap | Effort | Status |
|----|---------|---------------|-----|--------|--------|
| T1.1 | Dark void background | ~~No ClearColor set~~ | ~~Need near-black clear color, dim ambient~~ | 30 min | [x] |
| T1.2 | Rim lighting | ~~Single directional light only~~ | ~~Add back/rim light for dramatic edges~~ | 30 min | [x] |
| T1.3 | Table emissive boost | ~~Emissive values 0.01-0.08~~ | ~~Boost 2-3x when ambient is dimmed~~ | 30 min | [x] |
| T1.4 | Crystal nodes | ~~0% - Not implemented~~ | ~~Glowing nodes at each creature slot~~ | 2-3 hrs | [x] |
| T1.5 | Lane grid overlay | ~~Single divider line only~~ | ~~Add 4 vertical lane separators~~ | 1.5 hrs | [x] |
| T1.6 | Creature idle animation | ~~Static tokens~~ | ~~Add subtle bob/sway via sin(time)~~ | 1-2 hrs | [x] |
| T1.7 | Support faction materials | ~~Generic purple, no faction awareness~~ | ~~Copy creature pattern for faction colors~~ | 1-2 hrs | [x] |

**Estimated Tier 1 Total: 7-11 hours**

---

### Tier 2: High-Value Polish

| ID | Feature | Current State | Gap | Effort | Status |
|----|---------|---------------|-----|--------|--------|
| T2.1 | Visual health bars | ~~Text only~~ | ~~Horizontal bars with color gradient~~ | 2-3 hrs | [x] |
| T2.2 | Essence crystal display | ~~Text only~~ | ~~Filled/empty crystal pips~~ | 2 hrs | [x] |
| T2.3 | Creature ready-to-attack glow | ~~No indicator~~ | ~~3x emissive material when can attack~~ | 2 hrs | [x] |
| T2.4 | Creature exhausted state | ~~No indicator~~ | ~~Desaturated/dimmed material~~ | 1.5 hrs | [x] |
| T2.5 | Creature buff/debuff auras | ~~No indicator~~ | ~~Rotating torus rings (golden/purple)~~ | 2-3 hrs | [x] |
| T2.6 | Support durability pips | ~~Text only~~ | ~~Orbiting glowing spheres~~ | 1.5 hrs | [x] |
| T2.7 | Support ability trigger pulse | ~~No animation~~ | ~~Expanding/fading torus ring~~ | 1 hr | [x] |
| T2.8 | Point lights at creature slots | ~~No lighting~~ | ~~Faction-colored pulsing point lights~~ | 1 hr | [x] |

**Tier 2 Complete**

---

### Tier 3: Deferred to Phase 5A-4+

| ID | Feature | Notes | Phase |
|----|---------|-------|-------|
| T3.1 | Card art texture on gems | Requires art asset pipeline | 5A-4 |
| T3.2 | Parallax depth shader | Custom WGSL shader work | 5A-4 |
| T3.3 | Spawn particle effects | Wireframe → solidify animation | 5A-7 |
| T3.4 | Death shatter effects | Gem/plate fragment particles | 5A-7 |
| T3.5 | Trajectory arc on drag | Line from card to target slot | 5A-8 |
| T3.6 | Faction-specific support meshes | GLB model imports | 5A-4 |
| T3.7 | Creature damaged cracks | Procedural crack geometry | 5A-7 |

---

## Detailed Implementation Notes

### T1.1: Dark Void Background

**Location**: `src/rendering/lighting.rs` + `src/game/state.rs`

**Changes Required**:
1. Add `ClearColor` resource in `setup_game_resources()`:
   ```rust
   commands.insert_resource(ClearColor(Color::srgba(0.01, 0.01, 0.02, 1.0)));
   ```

2. Reduce ambient light in `setup_lighting()`:
   ```rust
   AmbientLight {
       color: Color::srgb(0.15, 0.15, 0.2),  // Dark blue tint
       brightness: 15.0,  // Down from 200
   }
   ```

**Verification**: Scene should feel like table floating in space, table glow becomes primary illumination.

---

### T1.2: Rim Lighting

**Location**: `src/rendering/lighting.rs`

**Changes Required**:
Add second directional light positioned behind/above:
```rust
// Rim light - creates edge highlights
commands.spawn((
    DirectionalLight {
        illuminance: 3000.0,
        shadows_enabled: false,
        color: Color::srgb(0.4, 0.35, 0.5),  // Cool purple tint
        ..default()
    },
    Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
        std::f32::consts::PI / 3.0,      // Pitch down
        -std::f32::consts::PI * 0.75,    // Behind-left
        0.0,
    )),
));
```

**Verification**: Creatures and table edges should have visible rim highlight separating them from void.

---

### T1.3: Table Emissive Boost

**Location**: `src/rendering/board.rs` - `create_table_material()` and `create_edge_material()`

**Changes Required**:
Multiply all emissive values by 2-3x:
- Argentum: `(0.05, 0.03, 0.01)` → `(0.12, 0.08, 0.03)`
- Symbiote: `(0.02, 0.08, 0.04)` → `(0.05, 0.20, 0.10)`
- Obsidion: `(0.06, 0.01, 0.03)` → `(0.15, 0.03, 0.08)`

Edge materials should be boosted similarly.

**Verification**: Table should visibly glow and illuminate nearby objects when ambient is dimmed.

---

### T1.4: Crystal Nodes

**Location**: New file `src/rendering/crystal_nodes.rs`

**Design Intent**: Each creature slot has a crystal node that serves as the "holographic projection source" for creatures.

**Implementation**:
1. Create `CrystalNode` component
2. Spawn 10 nodes (5 per player) at creature slot positions, slightly elevated (y=0.3)
3. Use small gem mesh from `meshes.rs`
4. Add pulsing animation (emissive intensity oscillates with sin(time))
5. Faction-colored materials matching player's faction

**Verification**: Nodes should pulse gently at each slot position, creating visual anchors.

---

### T1.5: Lane Grid Overlay

**Location**: `src/rendering/board.rs` - `spawn_board()`

**Changes Required**:
Add 4 vertical divider lines at lane boundaries:
```rust
// Lane separators (between lanes 1-2, 2-3, 3-4, 4-5)
let grid_material = materials.add(StandardMaterial {
    base_color: Color::srgba(0.6, 0.6, 0.7, 0.25),
    alpha_mode: AlphaMode::Blend,
    emissive: LinearRgba::new(0.1, 0.1, 0.15, 1.0),
    ..default()
});
let vertical_line = meshes.add(Cuboid::new(0.05, 0.03, 4.5));

for i in 0..4 {
    let x = -3.0 + i as f32 * 2.0;  // -3, -1, 1, 3
    commands.spawn((
        Mesh3d(vertical_line.clone()),
        MeshMaterial3d(grid_material.clone()),
        Transform::from_xyz(x, 0.03, 0.0),
        GameBoard,
    ));
}
```

**Verification**: Faint grid lines should divide the 5 lanes without being distracting.

---

### T1.6: Creature Idle Animation

**Location**: `src/rendering/creatures.rs`

**Changes Required**:
Add new system `animate_idle_creatures`:
```rust
fn animate_idle_creatures(
    time: Res<Time>,
    mut creatures: Query<(&mut Transform, &Creature3D)>,
) {
    for (mut transform, creature) in creatures.iter_mut() {
        let t = time.elapsed_secs();
        // Each creature has slightly different phase based on slot
        let phase = creature.slot as f32 * 0.5;
        let bob = (t * 1.5 + phase).sin() * 0.05;
        transform.translation.y = 0.6 + bob;  // Base height + bob
    }
}
```

Register in plugin's Update systems.

**Verification**: Creature tokens should gently bob up and down, feeling alive.

---

### T1.7: Support Faction Materials

**Location**: `src/rendering/supports.rs`

**Changes Required**:
1. Add `get_player_faction()` call in `setup_support_assets()`
2. Create `create_support_material(faction, is_player1)` following creature pattern
3. Faction colors:
   - **Argentum**: Brass frame (0.7, 0.55, 0.25), amber emissive
   - **Symbiote**: Organic green (0.25, 0.5, 0.35), green/purple emissive
   - **Obsidion**: Dark obsidian (0.15, 0.08, 0.12), crimson emissive

**Verification**: Support tokens should match their faction's visual identity like creatures do.

---

## Progress Log

| Date | Items Completed | Notes |
|------|-----------------|-------|
| 2026-01-20 | Audit completed | 5 explore agents analyzed codebase |
| 2026-01-20 | T1.1, T1.2 | Dark void + rim lighting in lighting.rs |
| 2026-01-20 | T1.3 | Table emissive boost in board.rs |
| 2026-01-20 | T1.5 | Lane grid overlay (4 vertical lines) |
| 2026-01-20 | T1.6 | Creature idle animation in creatures.rs |
| 2026-01-20 | T1.7 | Support faction materials in supports.rs |
| 2026-01-20 | T1.4 | Crystal nodes (new module crystal_nodes.rs) |
| 2026-01-20 | **Tier 1 Complete** | All 7 essential items implemented |
| 2026-01-20 | T2.1, T2.2 | Visual health bars + essence crystals in hud.rs |
| 2026-01-20 | T2.3, T2.4 | Ready-to-attack glow + exhausted state in creatures.rs |
| 2026-01-20 | T2.5 | Buff/debuff auras (rotating torus rings) in creatures.rs |
| 2026-01-20 | T2.6 | Support durability pips (orbiting spheres) in supports.rs |
| 2026-01-20 | T2.7 | Support trigger pulse (expanding torus) in supports.rs |
| 2026-01-20 | T2.8 | Point lights at creature slots in crystal_nodes.rs |
| 2026-01-20 | **Tier 2 Complete** | All 8 polish items implemented |
| 2026-01-20 | **Phase 5A-3 Complete** | 100% of planned items done |

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/rendering/lighting.rs` | T1.1 (ambient), T1.2 (rim light) |
| `src/game/state.rs` | T1.1 (ClearColor resource) |
| `src/rendering/board.rs` | T1.3 (emissive boost), T1.5 (grid) |
| `src/rendering/crystal_nodes.rs` | T1.4 (new file) |
| `src/rendering/creatures.rs` | T1.6 (idle animation) |
| `src/rendering/supports.rs` | T1.7 (faction materials) |
| `src/rendering/mod.rs` | Export crystal_nodes module |
| `src/ui/hud.rs` | T2.1, T2.2 (visual bars/crystals) |

---

## Success Criteria

Phase 5A-3 is complete when:

- [ ] Scene renders with dark void background (near-black)
- [ ] Table glows as primary light source (faction-colored)
- [ ] Rim lighting creates dramatic edge highlights
- [ ] Crystal nodes pulse at each creature slot
- [ ] Lane grid is visible but subtle
- [ ] Creature tokens bob/sway gently
- [ ] Support tokens have faction-specific materials
- [ ] Visual matches "Farsight Table in dark void" design intent

---

## References

- Design Document: `docs/phase-5a-design.md` (Sections 1.1-1.6, 2.2)
- Creature materials pattern: `src/rendering/creatures.rs:69-132`
- Gem mesh creation: `src/rendering/meshes.rs:75-209`
