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
                ui.label(format!("Max Depth: {}", snapshot.max_depth));
                ui.separator();

                ui.label(egui::RichText::new("Action Visits").strong());
                ui.add_space(5.0);

                // Sort children by visits
                let mut children = snapshot.children.clone();
                children.sort_by(|a, b| b.visits.cmp(&a.visits));

                let max_visits = children.first().map(|c| c.visits).unwrap_or(1) as f32;

                for child in children.iter().take(10) {
                    ui.horizontal(|ui| {
                        // Progress bar for visit proportion
                        let progress = child.visits as f32 / max_visits;
                        ui.add(egui::ProgressBar::new(progress)
                            .text(format!("{}: {} visits ({:.1}%)",
                                format_action(&child.action),
                                child.visits,
                                child.win_rate * 100.0
                            ))
                        );
                    });
                }

                ui.separator();

                // Selected action highlight
                ui.label(egui::RichText::new("Selected Action").strong());
                ui.label(format_action(&snapshot.selected_action));
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
        cardgame::actions::Action::PlayCard { hand_index, slot } => {
            format!("Play card {} to slot {:?}", hand_index, slot)
        }
        cardgame::actions::Action::Attack { attacker, defender } => {
            format!("Attack from {:?} to {:?}", attacker, defender)
        }
        cardgame::actions::Action::UseAbility { slot, ability_index, target } => {
            format!("Use ability {} of {:?} on {:?}", ability_index, slot, target)
        }
        cardgame::actions::Action::EndTurn => "End Turn".to_string(),
    }
}
