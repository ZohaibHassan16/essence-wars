//! Action probability display.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};
use super::GlassboxState;

/// Plugin for action probability visualization.
pub struct ActionProbsPlugin;

impl Plugin for ActionProbsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_action_probs.run_if(in_state(AppState::Playing)));
    }
}

/// Draw action probability bars.
fn draw_action_probs(
    mut contexts: EguiContexts,
    glassbox: Res<GlassboxState>,
    bridge: Option<Res<GameBridge>>,
) {
    if !glassbox.visible {
        return;
    }

    let Some(bridge) = bridge else { return };

    egui::Window::new("Top Actions")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .resizable(false)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            if let Some(snapshot) = bridge.get_mcts_snapshot() {
                // Get top 5 actions by win rate
                let mut actions: Vec<_> = snapshot.action_win_rates.iter().collect();
                actions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

                ui.label(egui::RichText::new("Win Rate by Action").strong());
                ui.separator();

                for (action, &win_rate) in actions.iter().take(5) {
                    let visits = snapshot.action_visits.get(*action).copied().unwrap_or(0);

                    ui.horizontal(|ui| {
                        // Color based on win rate
                        let color = if win_rate > 0.6 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else if win_rate > 0.4 {
                            egui::Color32::from_rgb(200, 200, 50)
                        } else {
                            egui::Color32::from_rgb(200, 50, 50)
                        };

                        ui.add(egui::ProgressBar::new(win_rate as f32)
                            .fill(color)
                            .text(format!("{:.0}%", win_rate * 100.0))
                        );
                    });

                    ui.label(format!("  {} ({} visits)", format_action_short(action), visits));
                    ui.add_space(2.0);
                }
            } else {
                ui.label("No action data");
            }
        });
}

/// Format an action for short display.
fn format_action_short(action: &cardgame::actions::Action) -> String {
    match action {
        cardgame::actions::Action::PlayCard { hand_index, slot, .. } => {
            format!("Play #{} -> {}", hand_index, slot)
        }
        cardgame::actions::Action::Attack { attacker_slot, target } => {
            match target {
                cardgame::actions::AttackTarget::Creature(s) => format!("Atk {} -> {}", attacker_slot, s),
                cardgame::actions::AttackTarget::Face => format!("Atk {} -> Face", attacker_slot),
            }
        }
        cardgame::actions::Action::UseAbility { creature_slot, ability_index, .. } => {
            format!("Ability {}:{}", creature_slot, ability_index)
        }
        cardgame::actions::Action::EndTurn => "End Turn".to_string(),
    }
}
