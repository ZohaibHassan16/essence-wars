//! Value estimate gauge visualization.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game::{AppState, GameBridge};
use super::GlassboxState;

/// Plugin for value gauge visualization.
pub struct ValueGaugePlugin;

impl Plugin for ValueGaugePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_value_gauge.run_if(in_state(AppState::Playing)));
    }
}

/// Draw the value estimate gauge.
fn draw_value_gauge(
    mut contexts: EguiContexts,
    glassbox: Res<GlassboxState>,
    bridge: Option<Res<GameBridge>>,
) {
    if !glassbox.visible {
        return;
    }

    let Some(bridge) = bridge else { return };

    egui::Window::new("Value Estimate")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 100.0])
        .resizable(false)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            let value = bridge.get_mcts_snapshot()
                .map(|s| s.root_value)
                .unwrap_or(0.0);

            // Convert from [0, 1] to [-1, 1] for display
            let display_value = (value - 0.5) * 2.0;

            ui.vertical(|ui| {
                ui.label("AI Confidence");

                // Vertical gauge
                let gauge_height = 200.0;
                let gauge_width = 40.0;

                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(gauge_width, gauge_height),
                    egui::Sense::hover()
                );

                let painter = ui.painter_at(rect);

                // Background
                painter.rect_filled(rect, 4.0, egui::Color32::DARK_GRAY);

                // Fill based on value
                let fill_height = ((display_value + 1.0) / 2.0) * gauge_height;
                let fill_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x, rect.max.y - fill_height),
                    rect.max,
                );

                let fill_color = if display_value > 0.0 {
                    egui::Color32::from_rgb(50, 200, 50) // Green for positive
                } else {
                    egui::Color32::from_rgb(200, 50, 50) // Red for negative
                };

                painter.rect_filled(fill_rect, 4.0, fill_color);

                // Center line
                let center_y = rect.center().y;
                painter.line_segment(
                    [egui::pos2(rect.min.x, center_y), egui::pos2(rect.max.x, center_y)],
                    egui::Stroke::new(2.0, egui::Color32::WHITE)
                );

                ui.add_space(10.0);
                ui.label(format!("{:+.1}%", display_value * 100.0));

                // Legend
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(50, 200, 50), "+");
                    ui.label("P1 favored");
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "-");
                    ui.label("P2 favored");
                });
            });
        });
}
