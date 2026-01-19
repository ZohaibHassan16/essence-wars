# Bevy Glassbox Implementation Guide

## Overview
Practical guide for understanding and extending the Glassbox AI visualization in essence-wars-3d.

## Module Structure
```
crates/essence-wars-3d/src/glassbox/
├── mod.rs           # GlassboxState resource, GlassboxPlugin
├── mcts_panel.rs    # MCTS tree visualization (right sidebar)
├── action_probs.rs  # Action probability bars (bottom-left)
└── value_gauge.rs   # Value estimate gauge (top-left)
```

## Core Resource: GlassboxState

```rust
#[derive(Resource, Default)]
pub struct GlassboxState {
    pub visible: bool,  // Toggle with 'G' key
}
```

## Data Source: GameBridge

```rust
// From crates/essence-wars-3d/src/game/bridge.rs
impl GameBridge {
    pub fn get_mcts_snapshot(&self) -> Option<&MctsTreeSnapshot> {
        self.last_decision.as_ref()
            .and_then(|d| d.mcts_snapshot.as_ref())
    }
}
```

## Panel Pattern

Each panel follows this pattern:

```rust
fn draw_panel(
    mut contexts: EguiContexts,
    glassbox: Res<GlassboxState>,
    bridge: Option<Res<GameBridge>>,
) {
    if !glassbox.visible { return; }
    let Some(bridge) = bridge else { return; };

    egui::Window::new("Panel Name")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .show(contexts.ctx_mut(), |ui| {
            if let Some(snapshot) = bridge.get_mcts_snapshot() {
                // Render visualization
            }
        });
}
```

## Key Types from cardgame::bots

```rust
pub struct MctsNodeStats {
    pub action: Action,
    pub visits: u32,
    pub win_rate: f32,
}

pub struct MctsTreeSnapshot {
    pub total_simulations: u32,
    pub children: Vec<MctsNodeStats>,
    pub selected_action: Action,
    pub root_value: f32,
}
```

## Adding a New Panel

1. Create new file in `glassbox/` module
2. Add system function following the panel pattern
3. Register in `GlassboxPlugin::build()`
4. Query `GlassboxState` for visibility
5. Query `GameBridge` for MCTS data

## Toggle Implementation

```rust
fn toggle_glassbox(
    keys: Res<ButtonInput<KeyCode>>,
    mut glassbox: ResMut<GlassboxState>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        glassbox.visible = !glassbox.visible;
    }
}
```
