//! MCTS tree visualization panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};
use super::GlassboxState;

/// Plugin for MCTS panel visualization.
pub struct MctsPanelPlugin;

impl Plugin for MctsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_mcts_panel.run_if(in_state(AppState::Playing)));
    }
}

/// Draw the MCTS decision tree panel.
fn draw_mcts_panel(
    mut contexts: EguiContexts,
    glassbox: Res<GlassboxState>,
    bridge: Option<Res<GameBridge>>,
) {
    if !glassbox.visible {
        return;
    }

    let Some(bridge) = bridge else { return };

    egui::SidePanel::right("mcts_panel")
        .min_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("MCTS Decision Tree");
            ui.separator();

            if let Some(snapshot) = bridge.get_mcts_snapshot() {
                ui.label(format!("Total Simulations: {}", snapshot.total_simulations));
                ui.label(format!("Root Value: {:.2}%", snapshot.root_value * 100.0));
                ui.separator();

                ui.label(egui::RichText::new("Action Visits").strong());
                ui.add_space(5.0);

                // Show top actions by visit count
                let mut actions: Vec<_> = snapshot.action_visits.iter().collect();
                actions.sort_by(|a, b| b.1.cmp(a.1));

                let max_visits = actions.first().map(|(_, &v)| v).unwrap_or(1) as f32;

                for (action, &visits) in actions.iter().take(10) {
                    let win_rate = snapshot.action_win_rates
                        .get(*action)
                        .copied()
                        .unwrap_or(0.0);

                    ui.horizontal(|ui| {
                        // Progress bar for visit proportion
                        let progress = visits as f32 / max_visits;
                        ui.add(egui::ProgressBar::new(progress)
                            .text(format!("{}: {} visits ({:.1}%)",
                                format_action(action),
                                visits,
                                win_rate * 100.0
                            ))
                        );
                    });
                }

                ui.separator();

                // Selected action highlight
                ui.label(egui::RichText::new("Selected Action").strong());
                ui.label(format!("{}", format_action(&snapshot.selected_action)));
            } else {
                ui.label("No MCTS data available");
                ui.label("(Waiting for AI decision...)");
            }

            ui.separator();
            ui.label(egui::RichText::new("Press 'G' to toggle").small());
        });
}

/// Format an action for display.
fn format_action(action: &cardgame::actions::Action) -> String {
    match action {
        cardgame::actions::Action::PlayCard { hand_index, slot, target } => {
            match target {
                Some(t) => format!("Play card {} to slot {} targeting {:?}", hand_index, slot, t),
                None => format!("Play card {} to slot {}", hand_index, slot),
            }
        }
        cardgame::actions::Action::Attack { attacker_slot, target } => {
            format!("Attack from slot {} to {:?}", attacker_slot, target)
        }
        cardgame::actions::Action::UseAbility { creature_slot, ability_index, target } => {
            match target {
                Some(t) => format!("Use ability {} of slot {} on {:?}", ability_index, creature_slot, t),
                None => format!("Use ability {} of slot {}", ability_index, creature_slot),
            }
        }
        cardgame::actions::Action::EndTurn => "End Turn".to_string(),
    }
}
