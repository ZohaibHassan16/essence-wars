//! World-space labels rendered as egui billboards.
//!
//! This module renders card names and stats above creatures and supports
//! by converting world coordinates to screen positions for egui windows.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::game::{AppState, GameBridge};
use crate::rendering::{Creature3D, GameCamera, Support3D};

/// Plugin for rendering world-space labels.
pub struct LabelsPlugin;

impl Plugin for LabelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (render_creature_labels, render_support_labels)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Render labels above creatures showing their name and stats.
fn render_creature_labels(
    mut contexts: EguiContexts,
    bridge: Res<GameBridge>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    creatures: Query<(&Creature3D, &Transform)>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    for (creature, transform) in creatures.iter() {
        // Get card info from database
        let card_name = bridge
            .card_db
            .get(cardgame::types::CardId(creature.card_id))
            .map(|c| c.name.as_str())
            .unwrap_or("Unknown");

        // Calculate label position (above the creature)
        let label_world_pos = transform.translation + Vec3::new(0.0, 1.0, 0.0);

        // Convert world position to screen position
        let Ok(screen_pos) = camera.world_to_viewport(camera_transform, label_world_pos) else {
            continue;
        };

        // Create a unique window ID for this creature
        let window_id = egui::Id::new(format!("creature_label_{}", creature.instance_id));

        // Determine colors based on owner
        let (bg_color, text_color) = if creature.owner == 0 {
            (egui::Color32::from_rgba_unmultiplied(30, 60, 120, 200), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgba_unmultiplied(120, 30, 30, 200), egui::Color32::WHITE)
        };

        // Calculate stat colors (green if buffed, red if damaged, white if normal)
        let attack_color = egui::Color32::WHITE;
        let health_color = if creature.health <= 2 {
            egui::Color32::from_rgb(255, 100, 100) // Red for low health
        } else {
            egui::Color32::WHITE
        };

        // Render the label as an egui Area at the screen position
        egui::Area::new(window_id)
            .fixed_pos(egui::pos2(screen_pos.x - 40.0, screen_pos.y - 35.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Card name (truncated if too long)
                            let display_name = if card_name.len() > 12 {
                                format!("{}...", &card_name[..10])
                            } else {
                                card_name.to_string()
                            };
                            ui.label(egui::RichText::new(display_name)
                                .color(text_color)
                                .size(10.0));

                            // Stats row: Attack / Health
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{}", creature.attack))
                                    .color(attack_color)
                                    .size(11.0)
                                    .strong());
                                ui.label(egui::RichText::new("/")
                                    .color(text_color)
                                    .size(11.0));
                                ui.label(egui::RichText::new(format!("{}", creature.health))
                                    .color(health_color)
                                    .size(11.0)
                                    .strong());
                            });
                        });
                    });
            });
    }
}

/// Render labels above supports showing their name and durability.
fn render_support_labels(
    mut contexts: EguiContexts,
    bridge: Res<GameBridge>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    supports: Query<(&Support3D, &Transform)>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let ctx = contexts.ctx_mut();

    for (support, transform) in supports.iter() {
        // Get card info from database
        let card_name = bridge
            .card_db
            .get(cardgame::types::CardId(support.card_id))
            .map(|c| c.name.as_str())
            .unwrap_or("Unknown");

        // Calculate label position (above the support)
        let label_world_pos = transform.translation + Vec3::new(0.0, 0.8, 0.0);

        // Convert world position to screen position
        let Ok(screen_pos) = camera.world_to_viewport(camera_transform, label_world_pos) else {
            continue;
        };

        // Create a unique window ID for this support
        let window_id = egui::Id::new(format!("support_label_{}_{}", support.owner, support.slot));

        // Support labels are purple-toned
        let (bg_color, text_color) = if support.owner == 0 {
            (egui::Color32::from_rgba_unmultiplied(60, 30, 90, 200), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgba_unmultiplied(90, 30, 60, 200), egui::Color32::WHITE)
        };

        // Durability color (red if low)
        let durability_color = if support.durability <= 1 {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(200, 200, 100)
        };

        // Render the label as an egui Area
        egui::Area::new(window_id)
            .fixed_pos(egui::pos2(screen_pos.x - 35.0, screen_pos.y - 30.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Card name (truncated)
                            let display_name = if card_name.len() > 10 {
                                format!("{}...", &card_name[..8])
                            } else {
                                card_name.to_string()
                            };
                            ui.label(egui::RichText::new(display_name)
                                .color(text_color)
                                .size(9.0));

                            // Durability indicator
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Dur:")
                                    .color(text_color)
                                    .size(9.0));
                                ui.label(egui::RichText::new(format!("{}", support.durability))
                                    .color(durability_color)
                                    .size(10.0)
                                    .strong());
                            });
                        });
                    });
            });
    }
}
