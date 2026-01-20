//! HUD (Heads-Up Display) for game information.
//!
//! Features visual health bars and essence crystal pips for better
//! game state visibility.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};

/// Plugin for the game HUD.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_hud.run_if(in_state(AppState::Playing)));
    }
}

/// Starting life total for health bar percentage calculation.
const STARTING_LIFE: i16 = 20;

/// Get health bar color based on health percentage.
fn health_bar_color(current: i16, max: i16) -> egui::Color32 {
    let ratio = (current as f32) / (max as f32);
    if ratio > 0.6 {
        egui::Color32::from_rgb(80, 200, 80) // Green
    } else if ratio > 0.3 {
        egui::Color32::from_rgb(220, 180, 50) // Yellow
    } else {
        egui::Color32::from_rgb(220, 60, 60) // Red
    }
}

/// Draw a visual health bar.
fn draw_health_bar(ui: &mut egui::Ui, current: i16, max: i16) {
    let bar_width = 100.0;
    let bar_height = 14.0;
    let ratio = (current as f32 / max as f32).clamp(0.0, 1.0);
    let color = health_bar_color(current, max);

    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(bar_width, bar_height),
        egui::Sense::hover(),
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Background (dark)
        painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(40, 40, 50));

        // Health fill
        let fill_width = rect.width() * ratio;
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(fill_width, rect.height()),
        );
        painter.rect_filled(fill_rect, 3.0, color);

        // Border
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 90)));

        // Text overlay
        let text = format!("{}/{}", current, max);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );
    }
}

/// Draw essence crystal pips.
fn draw_essence_crystals(ui: &mut egui::Ui, current: u8, max: u8) {
    let pip_size = 10.0;
    let pip_spacing = 3.0;
    let total_width = max as f32 * (pip_size + pip_spacing) - pip_spacing;

    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(total_width.max(60.0), pip_size + 4.0),
        egui::Sense::hover(),
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Draw each pip
        for i in 0..max {
            let x = rect.min.x + i as f32 * (pip_size + pip_spacing) + pip_size / 2.0;
            let y = rect.center().y;
            let center = egui::pos2(x, y);
            let radius = pip_size / 2.0;

            if i < current {
                // Filled crystal (bright cyan/blue)
                painter.circle_filled(center, radius, egui::Color32::from_rgb(80, 180, 255));
                // Inner glow
                painter.circle_filled(center, radius * 0.5, egui::Color32::from_rgb(200, 240, 255));
            } else {
                // Empty crystal (dark outline)
                painter.circle_filled(center, radius, egui::Color32::from_rgb(30, 35, 45));
                painter.circle_stroke(center, radius, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 70, 90)));
            }
        }
    }
}

/// Draw the game HUD using egui.
fn draw_hud(
    mut contexts: EguiContexts,
    bridge: Option<Res<GameBridge>>,
) {
    let Some(bridge) = bridge else { return };
    let Some(client) = &bridge.client else { return };
    let Some(state) = client.get_state() else { return };

    egui::TopBottomPanel::top("game_hud")
        .frame(egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 220))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0)))
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                // Player 1 info
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Player 1").strong().color(egui::Color32::from_rgb(100, 180, 255)));

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("♥").color(egui::Color32::from_rgb(255, 100, 100)));
                            draw_health_bar(ui, state.players[0].life, STARTING_LIFE);
                        });

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("◆").color(egui::Color32::from_rgb(80, 180, 255)));
                            draw_essence_crystals(ui, state.players[0].current_essence, state.players[0].max_essence);
                        });

                        ui.label(egui::RichText::new(format!("Hand: {}", state.players[0].hand.len()))
                            .small()
                            .color(egui::Color32::from_rgb(180, 180, 190)));
                    });
                });

                ui.add_space(20.0);

                // Turn info (centered)
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("Turn {}", state.current_turn))
                        .heading()
                        .color(egui::Color32::from_rgb(220, 220, 230)));

                    let (current_player, player_color) = if state.active_player == cardgame::types::PlayerId::PLAYER_ONE {
                        ("Player 1's Turn", egui::Color32::from_rgb(100, 180, 255))
                    } else {
                        ("Player 2's Turn", egui::Color32::from_rgb(255, 120, 120))
                    };
                    ui.label(egui::RichText::new(current_player).color(player_color));
                });

                ui.add_space(20.0);

                // Player 2 info
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Player 2").strong().color(egui::Color32::from_rgb(255, 120, 120)));

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("♥").color(egui::Color32::from_rgb(255, 100, 100)));
                            draw_health_bar(ui, state.players[1].life, STARTING_LIFE);
                        });

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("◆").color(egui::Color32::from_rgb(80, 180, 255)));
                            draw_essence_crystals(ui, state.players[1].current_essence, state.players[1].max_essence);
                        });

                        ui.label(egui::RichText::new(format!("Hand: {}", state.players[1].hand.len()))
                            .small()
                            .color(egui::Color32::from_rgb(180, 180, 190)));
                    });
                });
            });
        });
}
