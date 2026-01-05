// Editor UI module

pub mod console;
pub mod hierarchy;
pub mod inspector;
pub mod viewport;

use egui::Context;
use engine_scene::{entity::EntityId, scene::Scene};

/// UI state for the editor
pub struct EditorUi {
    pub selected_entity: Option<EntityId>,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_console: bool,
    pub console_messages: Vec<ConsoleMessage>,
}

#[derive(Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConsoleLevel {
    Info,
    Warning,
    Error,
}

impl EditorUi {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            show_hierarchy: true,
            show_inspector: true,
            show_console: true,
            console_messages: Vec::new(),
        }
    }

    pub fn log_info(&mut self, message: String) {
        self.console_messages.push(ConsoleMessage {
            level: ConsoleLevel::Info,
            message,
        });
    }

    pub fn log_warning(&mut self, message: String) {
        self.console_messages.push(ConsoleMessage {
            level: ConsoleLevel::Warning,
            message,
        });
    }

    pub fn log_error(&mut self, message: String) {
        self.console_messages.push(ConsoleMessage {
            level: ConsoleLevel::Error,
            message,
        });
    }

    /// Render the entire editor UI
    pub fn render(&mut self, ctx: &Context, scene: &mut Scene) {
        // Menu bar
        self.render_menu_bar(ctx, scene);

        // Left panel - Hierarchy
        if self.show_hierarchy {
            hierarchy::render_hierarchy_panel(ctx, scene, &mut self.selected_entity);
        }

        // Right panel - Inspector
        if self.show_inspector {
            inspector::render_inspector_panel(ctx, scene, &mut self.selected_entity);
        }

        // Bottom panel - Console
        if self.show_console {
            console::render_console_panel(ctx, &mut self.console_messages);
        }
    }

    fn render_menu_bar(&mut self, ctx: &Context, _scene: &mut Scene) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        self.log_info("New scene created".to_string());
                    }
                    if ui.button("Open Scene...").clicked() {
                        self.log_info("Open scene (not implemented)".to_string());
                    }
                    if ui.button("Save Scene").clicked() {
                        self.log_info("Save scene (not implemented)".to_string());
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        self.log_info("Exit clicked".to_string());
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.log_info("Undo (not implemented)".to_string());
                    }
                    if ui.button("Redo").clicked() {
                        self.log_info("Redo (not implemented)".to_string());
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_hierarchy, "Hierarchy");
                    ui.checkbox(&mut self.show_inspector, "Inspector");
                    ui.checkbox(&mut self.show_console, "Console");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.log_info("Game Engine Editor - Phase 5".to_string());
                    }
                });
            });
        });
    }
}

impl Default for EditorUi {
    fn default() -> Self {
        Self::new()
    }
}
