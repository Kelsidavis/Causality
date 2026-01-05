// Console panel - displays logs and messages

use super::{ConsoleLevel, ConsoleMessage};
use egui::{Color32, Context, ScrollArea};

pub fn render_console_panel(ctx: &Context, messages: &mut Vec<ConsoleMessage>) {
    egui::TopBottomPanel::bottom("console_panel")
        .default_height(200.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Console");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() {
                        messages.clear();
                    }
                });
            });
            ui.separator();

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in messages.iter() {
                        let (icon, color) = match msg.level {
                            ConsoleLevel::Info => ("ℹ", Color32::LIGHT_BLUE),
                            ConsoleLevel::Warning => ("⚠", Color32::YELLOW),
                            ConsoleLevel::Error => ("❌", Color32::RED),
                        };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, icon);
                            ui.label(&msg.message);
                        });
                    }
                });
        });
}
