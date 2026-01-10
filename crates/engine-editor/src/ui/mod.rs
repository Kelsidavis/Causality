// Editor UI module

pub mod console;
pub mod hierarchy;
pub mod inspector;
pub mod viewport;

use egui::Context;
use engine_scene::{entity::EntityId, scene::Scene};

// Re-export types for use in main.rs
pub use inspector::InspectorResult;
pub use hierarchy::{HierarchyAction, HierarchyState};

/// Combined result from all editor UI panels
#[derive(Default)]
pub struct EditorResult {
    pub inspector: InspectorResult,
    pub hierarchy: HierarchyAction,
    pub scene_modified: bool,
    pub undo_requested: bool,
    pub redo_requested: bool,
    pub scene_changed: bool, // True when scene loaded or new scene created
}

/// UI state for the editor
pub struct EditorUi {
    pub selected_entity: Option<EntityId>,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_console: bool,
    pub console_messages: Vec<ConsoleMessage>,
    pub show_save_dialog: bool,
    pub show_save_as_dialog: bool,
    pub show_load_dialog: bool,
    pub show_new_scene_confirm: bool,
    pub show_exit_confirm: bool,
    pub show_about_dialog: bool,
    pub save_path: String,
    pub load_path: String,
    pub current_scene_path: Option<String>,
    pub scene_modified: bool,
    pub exit_requested: bool,
    // Hierarchy panel state
    pub hierarchy_state: HierarchyState,
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
            show_save_as_dialog: false,
            show_load_dialog: false,
            show_new_scene_confirm: false,
            show_exit_confirm: false,
            show_about_dialog: false,
            save_path: "assets/scenes/saved_scene.ron".to_string(),
            load_path: "assets/scenes/castle.ron".to_string(),
            current_scene_path: None,
            scene_modified: false,
            exit_requested: false,
            hierarchy_state: HierarchyState::default(),
        }
    }

    pub fn mark_scene_modified(&mut self) {
        self.scene_modified = true;
    }

    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
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

    /// Render the entire editor UI and return change indicators
    pub fn render(&mut self, ctx: &Context, scene: &mut Scene, can_undo: bool, can_redo: bool) -> EditorResult {
        let mut result = EditorResult::default();

        // Menu bar
        self.render_menu_bar(ctx, scene, can_undo, can_redo, &mut result);

        // Left panel - Hierarchy
        if self.show_hierarchy {
            result.hierarchy = hierarchy::render_hierarchy_panel(
                ctx,
                scene,
                &mut self.selected_entity,
                &mut self.hierarchy_state,
            );
        }

        // Right panel - Inspector (captures changes)
        if self.show_inspector {
            result.inspector = inspector::render_inspector_panel(ctx, scene, &mut self.selected_entity);
        }

        // Bottom panel - Console
        if self.show_console {
            console::render_console_panel(ctx, &mut self.console_messages);
        }

        // Dialogs
        if self.show_save_dialog {
            self.render_save_dialog(ctx, scene);
        }

        if self.show_save_as_dialog {
            self.render_save_as_dialog(ctx, scene);
        }

        if self.show_load_dialog {
            self.render_load_dialog(ctx, scene, &mut result);
        }

        if self.show_new_scene_confirm {
            self.render_new_scene_confirm(ctx, scene, &mut result);
        }

        if self.show_exit_confirm {
            self.render_exit_confirm(ctx);
        }

        if self.show_about_dialog {
            self.render_about_dialog(ctx);
        }

        result
    }

    fn render_menu_bar(&mut self, ctx: &Context, _scene: &mut Scene, can_undo: bool, can_redo: bool, result: &mut EditorResult) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(egui::Button::new("New Scene").shortcut_text("Ctrl+N")).clicked() {
                        self.show_new_scene_confirm = true;
                    }

                    if ui.add(egui::Button::new("Open Scene...").shortcut_text("Ctrl+O")).clicked() {
                        self.show_load_dialog = true;
                    }

                    ui.separator();

                    let save_text = if let Some(ref path) = self.current_scene_path {
                        format!("Save {}",
                            std::path::Path::new(path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Scene"))
                    } else {
                        "Save Scene".to_string()
                    };

                    if ui.add(egui::Button::new(save_text).shortcut_text("Ctrl+S")).clicked() {
                        if let Some(ref path) = self.current_scene_path {
                            self.save_path = path.clone();
                            self.show_save_dialog = true;
                        } else {
                            self.show_save_as_dialog = true;
                        }
                    }

                    if ui.add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S")).clicked() {
                        self.show_save_as_dialog = true;
                    }

                    ui.separator();

                    if ui.add(egui::Button::new("Exit").shortcut_text("Alt+F4")).clicked() {
                        if self.scene_modified {
                            self.show_exit_confirm = true;
                        } else {
                            self.exit_requested = true;
                        }
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(can_undo, egui::Button::new("Undo").shortcut_text("Ctrl+Z")).clicked() {
                        result.undo_requested = true;
                        ui.close();
                    }
                    if ui.add_enabled(can_redo, egui::Button::new("Redo").shortcut_text("Ctrl+Y")).clicked() {
                        result.redo_requested = true;
                        ui.close();
                    }

                    ui.separator();

                    let has_selection = self.selected_entity.is_some();
                    if ui.add_enabled(has_selection, egui::Button::new("Duplicate").shortcut_text("Ctrl+D")).clicked() {
                        if let Some(entity_id) = self.selected_entity {
                            result.hierarchy.duplicate_entity = Some(entity_id);
                        }
                        ui.close();
                    }
                    if ui.add_enabled(has_selection, egui::Button::new("Delete").shortcut_text("Delete")).clicked() {
                        if let Some(entity_id) = self.selected_entity {
                            result.hierarchy.delete_entity = Some(entity_id);
                        }
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_hierarchy, "Hierarchy");
                    ui.checkbox(&mut self.show_inspector, "Inspector");
                    ui.checkbox(&mut self.show_console, "Console");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about_dialog = true;
                    }
                    if ui.button("Documentation").clicked() {
                        self.log_info("Opening documentation...".to_string());
                    }
                });
            });
        });
    }

    fn render_save_dialog(&mut self, ctx: &Context, scene: &mut Scene) {
        egui::Window::new("Save Scene")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Save Scene");
                ui.add_space(10.0);

                ui.label("File path:");
                ui.text_edit_singleline(&mut self.save_path);

                let is_valid = self.save_path.ends_with(".ron") && !self.save_path.is_empty();
                if !is_valid {
                    ui.colored_label(egui::Color32::RED, "⚠ Path must end with .ron");
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add_enabled(is_valid, egui::Button::new("Save")).clicked() {
                        match scene.save_to_file(&self.save_path) {
                            Ok(_) => {
                                self.log_info(format!("Scene saved to: {}", self.save_path));
                                self.current_scene_path = Some(self.save_path.clone());
                                self.scene_modified = false;
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

    fn render_save_as_dialog(&mut self, ctx: &Context, scene: &mut Scene) {
        egui::Window::new("Save Scene As")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Save Scene As");
                ui.add_space(10.0);

                ui.label("File path:");
                ui.text_edit_singleline(&mut self.save_path);

                ui.label("💡 Tip: Use descriptive names like 'my_level.ron'");

                let is_valid = self.save_path.ends_with(".ron") && !self.save_path.is_empty();
                if !is_valid {
                    ui.colored_label(egui::Color32::RED, "⚠ Path must end with .ron");
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add_enabled(is_valid, egui::Button::new("Save As")).clicked() {
                        match scene.save_to_file(&self.save_path) {
                            Ok(_) => {
                                self.log_info(format!("Scene saved to: {}", self.save_path));
                                self.current_scene_path = Some(self.save_path.clone());
                                self.scene_modified = false;
                                self.show_save_as_dialog = false;
                            }
                            Err(e) => {
                                self.log_error(format!("Failed to save scene: {}", e));
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_save_as_dialog = false;
                    }
                });
            });
    }

    fn render_load_dialog(&mut self, ctx: &Context, scene: &mut Scene, result: &mut EditorResult) {
        egui::Window::new("Load Scene")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Load Scene");
                ui.add_space(10.0);

                ui.label("File path:");
                ui.text_edit_singleline(&mut self.load_path);

                let exists = std::path::Path::new(&self.load_path).exists();
                let is_valid = self.load_path.ends_with(".ron") && !self.load_path.is_empty();

                if !is_valid {
                    ui.colored_label(egui::Color32::RED, "⚠ Path must end with .ron");
                } else if !exists {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ File does not exist");
                }

                ui.label("💡 Example: assets/scenes/castle.ron");

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add_enabled(is_valid && exists, egui::Button::new("Load")).clicked() {
                        match Scene::load_from_file(&self.load_path) {
                            Ok(loaded_scene) => {
                                *scene = loaded_scene;
                                self.log_info(format!("Scene loaded from: {}", self.load_path));
                                self.current_scene_path = Some(self.load_path.clone());
                                self.scene_modified = false;
                                self.show_load_dialog = false;
                                self.selected_entity = None;
                                result.scene_changed = true; // Signal to clear undo history
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

    fn render_new_scene_confirm(&mut self, ctx: &Context, scene: &mut Scene, result: &mut EditorResult) {
        egui::Window::new("New Scene")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Create New Scene");
                ui.add_space(10.0);

                if self.scene_modified {
                    ui.colored_label(egui::Color32::YELLOW,
                        "⚠ Warning: You have unsaved changes!");
                    ui.label("Creating a new scene will discard all unsaved changes.");
                } else {
                    ui.label("This will clear the current scene.");
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Create New Scene").clicked() {
                        *scene = Scene::new("Untitled Scene".to_string());
                        self.log_info("Created new scene".to_string());
                        self.current_scene_path = None;
                        self.scene_modified = false;
                        self.selected_entity = None;
                        self.show_new_scene_confirm = false;
                        result.scene_changed = true; // Signal to clear undo history
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_new_scene_confirm = false;
                    }
                });
            });
    }

    fn render_exit_confirm(&mut self, ctx: &Context) {
        egui::Window::new("Exit")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Exit Editor");
                ui.add_space(10.0);

                ui.colored_label(egui::Color32::YELLOW,
                    "⚠ You have unsaved changes!");
                ui.label("Are you sure you want to exit?");

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Exit Without Saving").clicked() {
                        self.exit_requested = true;
                        self.show_exit_confirm = false;
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_exit_confirm = false;
                    }
                });
            });
    }

    fn render_about_dialog(&mut self, ctx: &Context) {
        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Causality Engine Editor");
                ui.add_space(10.0);

                ui.label("Version: 0.1.0");
                ui.label("A modern 3D game engine built with Rust");
                ui.add_space(5.0);

                ui.separator();
                ui.add_space(5.0);

                ui.label("Features:");
                ui.label("  • Scene editing with hierarchy");
                ui.label("  • Component-based entity system");
                ui.label("  • Physics simulation with Rapier3D");
                ui.label("  • Scripting with Rhai");
                ui.label("  • AI-powered asset generation");
                ui.label("  • Hot-reload for assets and scripts");

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Built with:");
                    ui.hyperlink_to("wgpu", "https://wgpu.rs/");
                    ui.label("•");
                    ui.hyperlink_to("egui", "https://www.egui.rs/");
                    ui.label("•");
                    ui.hyperlink_to("Rapier", "https://rapier.rs/");
                });

                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    self.show_about_dialog = false;
                }
            });
    }
}

impl Default for EditorUi {
    fn default() -> Self {
        Self::new()
    }
}
