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
    pub show_save_dialog: bool,
    pub show_load_dialog: bool,
    pub save_path: String,
    pub load_path: String,
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
            show_save_dialog: false,
            show_load_dialog: false,
            save_path: "assets/scenes/saved_scene.ron".to_string(),
            load_path: "assets/scenes/castle.ron".to_string(),
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

        // Save dialog
        if self.show_save_dialog {
            self.render_save_dialog(ctx, scene);
        }

        // Load dialog
        if self.show_load_dialog {
            self.render_load_dialog(ctx, scene);
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
                        self.show_load_dialog = true;
                    }
                    if ui.button("Save Scene").clicked() {
                        self.show_save_dialog = true;
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

    fn render_save_dialog(&mut self, ctx: &Context, scene: &mut Scene) {
        egui::Window::new("Save Scene")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Scene file path:");
                ui.text_edit_singleline(&mut self.save_path);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        match scene.save_to_file(&self.save_path) {
                            Ok(_) => {
                                self.log_info(format!("Scene saved to: {}", self.save_path));
                                self.show_save_dialog = false;
                            }
                            Err(e) => {
                                self.log_error(format!("Failed to save scene: {}", e));
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_save_dialog = false;
                    }
                });
            });
    }

    fn render_load_dialog(&mut self, ctx: &Context, scene: &mut Scene) {
        egui::Window::new("Load Scene")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Scene file path:");
                ui.text_edit_singleline(&mut self.load_path);

                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        match Scene::load_from_file(&self.load_path) {
                            Ok(loaded_scene) => {
                                *scene = loaded_scene;
                                self.log_info(format!("Scene loaded from: {}", self.load_path));
                                self.show_load_dialog = false;
                                self.selected_entity = None; // Clear selection
                            }
                            Err(e) => {
                                self.log_error(format!("Failed to load scene: {}", e));
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_load_dialog = false;
                    }
                });
            });
    }
}

impl Default for EditorUi {
    fn default() -> Self {
        Self::new()
    }
}
