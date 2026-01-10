// Editor UI module

pub mod console;
pub mod hierarchy;
pub mod inspector;
pub mod viewport;

use std::collections::HashSet;
use egui::Context;
use engine_scene::{entity::EntityId, scene::Scene};

// Re-export types for use in main.rs
pub use inspector::{InspectorResult, InspectorState};
pub use hierarchy::{HierarchyAction, HierarchyState};

/// Brush action to apply in the scene
#[derive(Default)]
pub struct BrushAction {
    /// If Some, place foliage at this world position
    pub place_at: Option<glam::Vec3>,
    /// If Some, erase foliage near this world position
    pub erase_at: Option<glam::Vec3>,
}

/// Combined result from all editor UI panels
#[derive(Default)]
pub struct EditorResult {
    pub inspector: InspectorResult,
    pub hierarchy: HierarchyAction,
    pub brush: BrushAction,
    pub scene_modified: bool,
    pub undo_requested: bool,
    pub redo_requested: bool,
    pub scene_changed: bool, // True when scene loaded or new scene created
    pub open_recent_file: Option<String>, // Path to recent file to open
}

/// Brush tool mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushMode {
    Select,         // Default selection mode
    Place,          // Place vegetation
    Erase,          // Remove vegetation
    TerrainRaise,   // Raise terrain height
    TerrainLower,   // Lower terrain height
    TerrainSmooth,  // Smooth terrain
    TerrainFlatten, // Flatten terrain to uniform height
}

impl BrushMode {
    /// Returns the terrain brush mode code (for heightmap apply_brush)
    pub fn terrain_mode_code(&self) -> Option<u8> {
        match self {
            BrushMode::TerrainRaise => Some(0),
            BrushMode::TerrainLower => Some(1),
            BrushMode::TerrainSmooth => Some(2),
            BrushMode::TerrainFlatten => Some(3),
            _ => None,
        }
    }

    /// Check if this is a terrain sculpting mode
    pub fn is_terrain_mode(&self) -> bool {
        matches!(self, BrushMode::TerrainRaise | BrushMode::TerrainLower | BrushMode::TerrainSmooth | BrushMode::TerrainFlatten)
    }

    /// Check if this is a vegetation mode
    pub fn is_vegetation_mode(&self) -> bool {
        matches!(self, BrushMode::Place | BrushMode::Erase)
    }
}

impl Default for BrushMode {
    fn default() -> Self {
        BrushMode::Select
    }
}

/// Vegetation type for brush placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VegetationType {
    PineTree,
    OakTree,
    Bush,
    Shrub,
}

impl VegetationType {
    pub fn all() -> &'static [VegetationType] {
        &[
            VegetationType::PineTree,
            VegetationType::OakTree,
            VegetationType::Bush,
            VegetationType::Shrub,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            VegetationType::PineTree => "Pine Tree",
            VegetationType::OakTree => "Oak Tree",
            VegetationType::Bush => "Bush",
            VegetationType::Shrub => "Shrub",
        }
    }

    pub fn mesh_name(&self) -> &'static str {
        match self {
            VegetationType::PineTree => "vegetation_pine",
            VegetationType::OakTree => "vegetation_oak",
            VegetationType::Bush => "vegetation_bush",
            VegetationType::Shrub => "vegetation_shrub",
        }
    }
}

/// Brush tool state for painting vegetation and sculpting terrain
#[derive(Debug, Clone)]
pub struct BrushTool {
    pub mode: BrushMode,
    pub vegetation_type: VegetationType,
    pub radius: f32,
    pub density: f32,        // Instances per brush stroke
    pub scale_min: f32,
    pub scale_max: f32,
    pub random_rotation: bool,
    // Terrain sculpting settings
    pub terrain_strength: f32,   // How fast terrain is modified
    pub terrain_hardness: f32,   // Edge falloff (0=soft, 1=hard)
}

impl Default for BrushTool {
    fn default() -> Self {
        Self {
            mode: BrushMode::Select,
            vegetation_type: VegetationType::PineTree,
            radius: 5.0,
            density: 2.0,
            scale_min: 0.5,  // More size variation
            scale_max: 1.5,
            random_rotation: true,
            terrain_strength: 1.0,
            terrain_hardness: 0.5,
        }
    }
}

/// Performance metrics for status bar
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub last_frame_times: Vec<f32>,
    pub last_update: std::time::Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            last_frame_times: Vec::with_capacity(60),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_update).as_secs_f32() * 1000.0;
        self.last_update = now;

        self.last_frame_times.push(frame_time);
        if self.last_frame_times.len() > 60 {
            self.last_frame_times.remove(0);
        }

        if !self.last_frame_times.is_empty() {
            let avg_frame_time: f32 = self.last_frame_times.iter().sum::<f32>() / self.last_frame_times.len() as f32;
            self.frame_time_ms = avg_frame_time;
            self.fps = if avg_frame_time > 0.0 { 1000.0 / avg_frame_time } else { 0.0 };
        }
    }
}

/// UI state for the editor
pub struct EditorUi {
    pub selected_entity: Option<EntityId>,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_console: bool,
    pub show_brush_panel: bool,
    pub show_shortcuts_help: bool,
    pub show_statistics: bool,
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
    pub recent_files: Vec<String>,
    pub max_recent_files: usize,
    pub scene_modified: bool,
    pub exit_requested: bool,
    // Hierarchy panel state
    pub hierarchy_state: HierarchyState,
    // Brush tool state
    pub brush_tool: BrushTool,
    // Inspector state (snapping settings)
    pub inspector_state: InspectorState,
    // Performance metrics
    pub performance: PerformanceMetrics,
    // Camera info for status bar
    pub camera_position: glam::Vec3,
    pub camera_distance: f32,
    // Undo/redo info for statistics
    pub undo_count: usize,
    pub redo_count: usize,
    // Hidden entities (editor-only, not saved to scene)
    pub hidden_entities: HashSet<EntityId>,
    // Locked entities (prevent modification)
    pub locked_entities: HashSet<EntityId>,
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
            show_brush_panel: false,
            show_shortcuts_help: false,
            show_statistics: false,
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
            recent_files: Vec::new(),
            max_recent_files: 10,
            scene_modified: false,
            exit_requested: false,
            hierarchy_state: HierarchyState::default(),
            brush_tool: BrushTool::default(),
            inspector_state: InspectorState::default(),
            performance: PerformanceMetrics::new(),
            camera_position: glam::Vec3::ZERO,
            camera_distance: 15.0,
            undo_count: 0,
            redo_count: 0,
            hidden_entities: HashSet::new(),
            locked_entities: HashSet::new(),
        }
    }

    /// Toggle visibility of an entity
    pub fn toggle_entity_visibility(&mut self, entity_id: EntityId) {
        if self.hidden_entities.contains(&entity_id) {
            self.hidden_entities.remove(&entity_id);
        } else {
            self.hidden_entities.insert(entity_id);
        }
    }

    /// Check if an entity is visible
    pub fn is_entity_visible(&self, entity_id: EntityId) -> bool {
        !self.hidden_entities.contains(&entity_id)
    }

    /// Show all hidden entities
    pub fn show_all_entities(&mut self) {
        self.hidden_entities.clear();
    }

    /// Hide all entities except the specified one
    pub fn hide_all_except(&mut self, scene: &Scene, entity_id: EntityId) {
        self.hidden_entities.clear();
        for entity in scene.entities() {
            if entity.id != entity_id {
                self.hidden_entities.insert(entity.id);
            }
        }
    }

    /// Get number of hidden entities
    pub fn hidden_count(&self) -> usize {
        self.hidden_entities.len()
    }

    /// Toggle lock state of an entity
    pub fn toggle_entity_lock(&mut self, entity_id: EntityId) {
        if self.locked_entities.contains(&entity_id) {
            self.locked_entities.remove(&entity_id);
        } else {
            self.locked_entities.insert(entity_id);
        }
    }

    /// Check if an entity is locked
    pub fn is_entity_locked(&self, entity_id: EntityId) -> bool {
        self.locked_entities.contains(&entity_id)
    }

    /// Unlock all entities
    pub fn unlock_all_entities(&mut self) {
        self.locked_entities.clear();
    }

    /// Get number of locked entities
    pub fn locked_count(&self) -> usize {
        self.locked_entities.len()
    }

    /// Update camera info for status bar display
    pub fn update_camera_info(&mut self, position: glam::Vec3, distance: f32) {
        self.camera_position = position;
        self.camera_distance = distance;
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

    /// Add a file to the recent files list
    pub fn add_recent_file(&mut self, path: String) {
        // Remove if already exists (to move it to the front)
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Trim to max
        self.recent_files.truncate(self.max_recent_files);
    }

    /// Render the entire editor UI and return change indicators
    pub fn render(&mut self, ctx: &Context, scene: &mut Scene, can_undo: bool, can_redo: bool, undo_count: usize, redo_count: usize) -> EditorResult {
        let mut result = EditorResult::default();

        // Update performance metrics
        self.performance.update();

        // Store undo/redo counts for statistics
        self.undo_count = undo_count;
        self.redo_count = redo_count;

        // Menu bar
        self.render_menu_bar(ctx, scene, can_undo, can_redo, &mut result);

        // Status bar at the bottom (before other panels to reserve space)
        self.render_status_bar(ctx, scene);

        // Left panel - Hierarchy
        if self.show_hierarchy {
            result.hierarchy = hierarchy::render_hierarchy_panel(
                ctx,
                scene,
                &mut self.selected_entity,
                &mut self.hierarchy_state,
                &self.hidden_entities,
                &self.locked_entities,
            );
        }

        // Right panel - Inspector (captures changes)
        if self.show_inspector {
            let is_locked = self.selected_entity
                .map(|id| self.locked_entities.contains(&id))
                .unwrap_or(false);
            result.inspector = inspector::render_inspector_panel(ctx, scene, &mut self.selected_entity, &mut self.inspector_state, is_locked);
        }

        // Bottom panel - Console
        if self.show_console {
            console::render_console_panel(ctx, &mut self.console_messages);
        }

        // Brush tool panel (floating window)
        if self.show_brush_panel {
            self.render_brush_panel(ctx);
        }

        // Statistics window
        if self.show_statistics {
            self.render_statistics_window(ctx, scene);
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

        if self.show_shortcuts_help {
            self.render_shortcuts_help(ctx);
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

                    // Recent Files submenu
                    ui.menu_button("Recent Files", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("No recent files");
                        } else {
                            for path in self.recent_files.clone() {
                                let file_name = std::path::Path::new(&path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&path);
                                if ui.button(file_name).clicked() {
                                    result.open_recent_file = Some(path);
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button("Clear Recent Files").clicked() {
                                self.recent_files.clear();
                                ui.close();
                            }
                        }
                    });

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
                    if ui.checkbox(&mut self.show_hierarchy, "Hierarchy").changed() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_inspector, "Inspector").changed() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_console, "Console").changed() {
                        ui.close();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_brush_panel, "Brush Tool").changed() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_statistics, "Statistics").changed() {
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        self.show_hierarchy = true;
                        self.show_inspector = true;
                        self.show_console = true;
                        self.show_brush_panel = false;
                        self.show_statistics = false;
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.add(egui::Button::new("Keyboard Shortcuts").shortcut_text("F1")).clicked() {
                        self.show_shortcuts_help = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("About").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
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
                                self.add_recent_file(self.save_path.clone());
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
                                self.add_recent_file(self.save_path.clone());
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
                                self.add_recent_file(self.load_path.clone());
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

    fn render_brush_panel(&mut self, ctx: &Context) {
        egui::Window::new("Brush Tool")
            .default_width(220.0)
            .resizable(true)
            .show(ctx, |ui| {
                // Tool category tabs
                ui.heading("Tool Category");
                ui.horizontal(|ui| {
                    let is_vegetation = self.brush_tool.mode.is_vegetation_mode() || self.brush_tool.mode == BrushMode::Select;
                    let is_terrain = self.brush_tool.mode.is_terrain_mode();

                    if ui.selectable_label(!is_terrain, "Vegetation").clicked() && is_terrain {
                        self.brush_tool.mode = BrushMode::Select;
                    }
                    if ui.selectable_label(is_terrain, "Terrain").clicked() && !is_terrain {
                        self.brush_tool.mode = BrushMode::TerrainRaise;
                    }
                });

                ui.separator();

                // Mode selection based on category
                if self.brush_tool.mode.is_terrain_mode() {
                    // Terrain sculpting modes
                    ui.heading("Sculpt Mode");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::TerrainRaise, "Raise");
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::TerrainLower, "Lower");
                    });
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::TerrainSmooth, "Smooth");
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::TerrainFlatten, "Flatten");
                    });

                    ui.separator();
                    ui.heading("Brush Settings");

                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(&mut self.brush_tool.radius)
                            .range(1.0..=30.0)
                            .speed(0.2));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Strength:");
                        ui.add(egui::DragValue::new(&mut self.brush_tool.terrain_strength)
                            .range(0.1..=5.0)
                            .speed(0.05));
                    });

                    ui.separator();

                    // Mode-specific tips
                    match self.brush_tool.mode {
                        BrushMode::TerrainRaise => {
                            ui.label("Click and drag to raise terrain");
                        }
                        BrushMode::TerrainLower => {
                            ui.label("Click and drag to lower terrain");
                        }
                        BrushMode::TerrainSmooth => {
                            ui.label("Click and drag to smooth bumps");
                        }
                        BrushMode::TerrainFlatten => {
                            ui.label("Click and drag to flatten to level");
                        }
                        _ => {}
                    }
                } else {
                    // Vegetation modes
                    ui.heading("Tool Mode");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::Select, "Select");
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::Place, "Place");
                        ui.selectable_value(&mut self.brush_tool.mode, BrushMode::Erase, "Erase");
                    });

                    ui.separator();

                    // Only show vegetation settings when in Place mode
                    if self.brush_tool.mode == BrushMode::Place {
                        ui.heading("Vegetation Type");
                        egui::ComboBox::from_label("")
                            .selected_text(self.brush_tool.vegetation_type.name())
                            .show_ui(ui, |ui| {
                                for veg_type in VegetationType::all() {
                                    ui.selectable_value(
                                        &mut self.brush_tool.vegetation_type,
                                        *veg_type,
                                        veg_type.name(),
                                    );
                                }
                            });

                        ui.separator();
                    }

                    if self.brush_tool.mode != BrushMode::Select {
                        ui.heading("Brush Settings");

                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut self.brush_tool.radius)
                                .range(0.5..=20.0)
                                .speed(0.1));
                        });

                        if self.brush_tool.mode == BrushMode::Place {
                            ui.horizontal(|ui| {
                                ui.label("Density:");
                                ui.add(egui::DragValue::new(&mut self.brush_tool.density)
                                    .range(1.0..=20.0)
                                    .speed(0.1));
                            });

                            ui.separator();
                            ui.heading("Scale");

                            ui.horizontal(|ui| {
                                ui.label("Min:");
                                ui.add(egui::DragValue::new(&mut self.brush_tool.scale_min)
                                    .range(0.1..=3.0)
                                    .speed(0.05));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Max:");
                                ui.add(egui::DragValue::new(&mut self.brush_tool.scale_max)
                                    .range(0.1..=3.0)
                                    .speed(0.05));
                            });

                            ui.checkbox(&mut self.brush_tool.random_rotation, "Random Rotation");
                        }
                    }

                    ui.separator();

                    // Instructions
                    if self.brush_tool.mode != BrushMode::Select {
                        ui.label("Left-click on terrain to paint");
                        if self.brush_tool.mode == BrushMode::Place {
                            ui.label("Hold Shift + click to erase");
                        }
                    }
                }
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

    fn render_status_bar(&self, ctx: &Context, scene: &Scene) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // FPS and frame time
                let fps_color = if self.performance.fps >= 55.0 {
                    egui::Color32::GREEN
                } else if self.performance.fps >= 30.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                ui.colored_label(fps_color, format!("FPS: {:.0}", self.performance.fps));
                ui.separator();
                ui.label(format!("{:.1}ms", self.performance.frame_time_ms));
                ui.separator();

                // Scene info (with hidden count)
                let hidden = self.hidden_count();
                if hidden > 0 {
                    ui.label(format!("Entities: {} ({} hidden)", scene.entity_count(), hidden));
                } else {
                    ui.label(format!("Entities: {}", scene.entity_count()));
                }
                ui.separator();

                // Camera position
                ui.label(format!(
                    "Camera: ({:.1}, {:.1}, {:.1})",
                    self.camera_position.x,
                    self.camera_position.y,
                    self.camera_position.z
                ));
                ui.separator();

                // Selected entity indicator or help tip
                if let Some(entity_id) = self.selected_entity {
                    if let Some(entity) = scene.get_entity(entity_id) {
                        ui.colored_label(egui::Color32::from_rgb(255, 200, 100), format!("Selected: {}", entity.name));
                        ui.separator();
                    }
                } else if self.brush_tool.mode == BrushMode::Select {
                    ui.colored_label(egui::Color32::GRAY, "Click viewport to select | F1 for shortcuts");
                    ui.separator();
                }

                // Brush mode indicator with radius
                if self.brush_tool.mode != BrushMode::Select {
                    let mode_text = match self.brush_tool.mode {
                        BrushMode::Place => format!("Brush: Place {}", self.brush_tool.vegetation_type.name()),
                        BrushMode::Erase => "Brush: Erase".to_string(),
                        BrushMode::TerrainRaise => "Terrain: Raise".to_string(),
                        BrushMode::TerrainLower => "Terrain: Lower".to_string(),
                        BrushMode::TerrainSmooth => "Terrain: Smooth".to_string(),
                        BrushMode::TerrainFlatten => "Terrain: Flatten".to_string(),
                        BrushMode::Select => "".to_string(),
                    };
                    let color = if self.brush_tool.mode.is_terrain_mode() {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::LIGHT_BLUE
                    };
                    ui.colored_label(color, format!("{} (r={:.1})", mode_text, self.brush_tool.radius));
                    ui.separator();
                }

                // Modified indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.scene_modified {
                        ui.colored_label(egui::Color32::YELLOW, "Modified");
                    } else {
                        ui.label("Saved");
                    }

                    // Scene name
                    if let Some(ref path) = self.current_scene_path {
                        ui.separator();
                        let file_name = std::path::Path::new(path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");
                        ui.label(file_name);
                    }
                });
            });
        });
    }

    fn render_statistics_window(&self, ctx: &Context, scene: &Scene) {
        egui::Window::new("Statistics")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Performance");
                ui.horizontal(|ui| {
                    ui.label("FPS:");
                    let fps_color = if self.performance.fps >= 55.0 {
                        egui::Color32::GREEN
                    } else if self.performance.fps >= 30.0 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(fps_color, format!("{:.1}", self.performance.fps));
                });
                ui.horizontal(|ui| {
                    ui.label("Frame Time:");
                    ui.label(format!("{:.2} ms", self.performance.frame_time_ms));
                });

                ui.separator();
                ui.heading("Scene");
                ui.horizontal(|ui| {
                    ui.label("Entities:");
                    ui.label(format!("{}", scene.entity_count()));
                });

                // Count components
                let mut mesh_count = 0;
                let mut light_count = 0;
                let mut particle_count = 0;
                let mut foliage_instances = 0;

                use engine_scene::components::{MeshRenderer, Light, ParticleEmitter, Foliage};
                for entity in scene.entities() {
                    if entity.has_component::<MeshRenderer>() {
                        mesh_count += 1;
                    }
                    if entity.has_component::<Light>() {
                        light_count += 1;
                    }
                    if entity.has_component::<ParticleEmitter>() {
                        particle_count += 1;
                    }
                    if let Some(foliage) = entity.get_component::<Foliage>() {
                        foliage_instances += foliage.instances.len();
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Meshes:");
                    ui.label(format!("{}", mesh_count));
                });
                ui.horizontal(|ui| {
                    ui.label("Lights:");
                    ui.label(format!("{}", light_count));
                });
                ui.horizontal(|ui| {
                    ui.label("Particle Emitters:");
                    ui.label(format!("{}", particle_count));
                });
                ui.horizontal(|ui| {
                    ui.label("Foliage Instances:");
                    ui.label(format!("{}", foliage_instances));
                });

                ui.separator();
                ui.heading("Camera");
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.label(format!(
                        "({:.1}, {:.1}, {:.1})",
                        self.camera_position.x,
                        self.camera_position.y,
                        self.camera_position.z
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Distance:");
                    ui.label(format!("{:.1}", self.camera_distance));
                });

                ui.separator();
                ui.heading("History");
                ui.horizontal(|ui| {
                    ui.label("Undo Stack:");
                    ui.label(format!("{}", self.undo_count));
                });
                ui.horizontal(|ui| {
                    ui.label("Redo Stack:");
                    ui.label(format!("{}", self.redo_count));
                });

                ui.separator();
                ui.heading("Session");
                ui.horizontal(|ui| {
                    ui.label("Recent Files:");
                    ui.label(format!("{}", self.recent_files.len()));
                });
            });
    }

    fn render_shortcuts_help(&mut self, ctx: &Context) {
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("File Operations");
                egui::Grid::new("file_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("Ctrl+N");
                    ui.label("New Scene");
                    ui.end_row();
                    ui.label("Ctrl+O");
                    ui.label("Open Scene");
                    ui.end_row();
                    ui.label("Ctrl+S");
                    ui.label("Save Scene");
                    ui.end_row();
                    ui.label("Ctrl+Shift+S");
                    ui.label("Save Scene As");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("Edit Operations");
                egui::Grid::new("edit_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("Ctrl+Z");
                    ui.label("Undo");
                    ui.end_row();
                    ui.label("Ctrl+Y / Ctrl+Shift+Z");
                    ui.label("Redo");
                    ui.end_row();
                    ui.label("Ctrl+C");
                    ui.label("Copy Entity");
                    ui.end_row();
                    ui.label("Ctrl+V");
                    ui.label("Paste Entity");
                    ui.end_row();
                    ui.label("Ctrl+D");
                    ui.label("Duplicate Entity");
                    ui.end_row();
                    ui.label("Delete");
                    ui.label("Delete Selected Entity");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("Viewport Controls");
                egui::Grid::new("viewport_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("F");
                    ui.label("Focus on Selected Entity");
                    ui.end_row();
                    ui.label("H");
                    ui.label("Toggle Entity Visibility");
                    ui.end_row();
                    ui.label("Shift+H");
                    ui.label("Show All Hidden Entities");
                    ui.end_row();
                    ui.label("Alt+H");
                    ui.label("Hide All Except Selected");
                    ui.end_row();
                    ui.label("L");
                    ui.label("Toggle Entity Lock");
                    ui.end_row();
                    ui.label("Home");
                    ui.label("Reset Camera View");
                    ui.end_row();
                    ui.label("Numpad 1");
                    ui.label("Front View");
                    ui.end_row();
                    ui.label("Numpad 3");
                    ui.label("Right View");
                    ui.end_row();
                    ui.label("Numpad 7");
                    ui.label("Top View");
                    ui.end_row();
                    ui.label("Right Mouse Drag");
                    ui.label("Orbit Camera");
                    ui.end_row();
                    ui.label("Middle Mouse Drag");
                    ui.label("Pan Camera");
                    ui.end_row();
                    ui.label("Mouse Wheel");
                    ui.label("Zoom In/Out");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("Hierarchy");
                egui::Grid::new("hierarchy_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("Up/Down");
                    ui.label("Navigate Entities");
                    ui.end_row();
                    ui.label("Left/Right");
                    ui.label("Collapse/Expand Node");
                    ui.end_row();
                    ui.label("P");
                    ui.label("Select Parent Entity");
                    ui.end_row();
                    ui.label("[");
                    ui.label("Select First Child");
                    ui.end_row();
                    ui.label("]");
                    ui.label("Select Next Sibling");
                    ui.end_row();
                    ui.label("Shift+]");
                    ui.label("Select Previous Sibling");
                    ui.end_row();
                    ui.label("Ctrl+Shift+N");
                    ui.label("Create New Entity");
                    ui.end_row();
                    ui.label("F2");
                    ui.label("Rename Selected Entity");
                    ui.end_row();
                    ui.label("Delete");
                    ui.label("Delete Selected Entity");
                    ui.end_row();
                    ui.label("Double-click");
                    ui.label("Rename Entity in Hierarchy");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("Panels");
                egui::Grid::new("panel_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("F1");
                    ui.label("Toggle Shortcuts Help");
                    ui.end_row();
                    ui.label("F3");
                    ui.label("Toggle Statistics Panel");
                    ui.end_row();
                    ui.label("F4");
                    ui.label("Toggle Console Panel");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("Brush Tool");
                egui::Grid::new("brush_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("Shift+Scroll");
                    ui.label("Adjust Brush Size");
                    ui.end_row();
                    ui.label("Left Click + Drag");
                    ui.label("Paint/Sculpt");
                    ui.end_row();
                });

                ui.separator();
                ui.heading("General");
                egui::Grid::new("general_shortcuts").striped(true).show(ui, |ui| {
                    ui.label("Escape");
                    ui.label("Deselect Entity / Exit Editor");
                    ui.end_row();
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.show_shortcuts_help = false;
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
