// Hierarchy panel - shows scene graph tree

use egui::{Context, ScrollArea};
use engine_scene::{entity::EntityId, scene::Scene};

/// Actions returned from the hierarchy panel
#[derive(Default)]
pub struct HierarchyAction {
    pub create_entity: Option<(String, Option<EntityId>)>,  // (name, parent)
    pub delete_entity: Option<EntityId>,
    pub duplicate_entity: Option<EntityId>,
    pub reparent: Option<(EntityId, Option<EntityId>)>,     // (child, new_parent)
    pub rename_entity: Option<(EntityId, String)>,          // (entity, new_name)
}

/// State for hierarchy UI (stored in EditorUi)
pub struct HierarchyState {
    pub show_create_dialog: bool,
    pub new_entity_name: String,
    pub new_entity_parent: Option<EntityId>,
    pub show_delete_confirm: bool,
    pub entity_to_delete: Option<EntityId>,
    pub context_menu_entity: Option<EntityId>,
    /// Search filter for entities
    pub search_filter: String,
    /// Expanded entity nodes (for tree view)
    pub expanded_entities: std::collections::HashSet<EntityId>,
    /// Entity currently being renamed (double-click to edit)
    pub editing_entity: Option<EntityId>,
    /// Temporary name while editing
    pub editing_name: String,
}

impl Default for HierarchyState {
    fn default() -> Self {
        Self {
            show_create_dialog: false,
            new_entity_name: "New Entity".to_string(),
            new_entity_parent: None,
            show_delete_confirm: false,
            entity_to_delete: None,
            context_menu_entity: None,
            search_filter: String::new(),
            expanded_entities: std::collections::HashSet::new(),
            editing_entity: None,
            editing_name: String::new(),
        }
    }
}

pub fn render_hierarchy_panel(
    ctx: &Context,
    scene: &Scene,
    selected_entity: &mut Option<EntityId>,
    state: &mut HierarchyState,
) -> HierarchyAction {
    let mut action = HierarchyAction::default();

    egui::SidePanel::left("hierarchy_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            // Search box
            ui.horizontal(|ui| {
                ui.label("Search:");
                let response = ui.text_edit_singleline(&mut state.search_filter);
                if response.changed() && !state.search_filter.is_empty() {
                    // Auto-expand all matches when searching
                    expand_matching_entities(scene, &state.search_filter, &mut state.expanded_entities);
                }
                if ui.small_button("X").clicked() {
                    state.search_filter.clear();
                }
            });

            // Show entity count and match count
            let total_count = scene.entity_count();
            if state.search_filter.is_empty() {
                ui.label(format!("{} entities", total_count));
            } else {
                let match_count = count_matching_entities(scene, &state.search_filter);
                ui.label(format!("{} / {} entities", match_count, total_count));
            }

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                // Show all root entities (entities without parents)
                for entity in scene.entities() {
                    if entity.parent.is_none() {
                        render_entity_tree(ui, scene, entity.id, selected_entity, state, &mut action, 0);
                    }
                }
            });

            ui.separator();

            // Create entity button
            if ui.button("+ Create Entity").clicked() {
                state.show_create_dialog = true;
                state.new_entity_parent = None;
                state.new_entity_name = "New Entity".to_string();
            }

            // Delete selected button (only show if something selected)
            if let Some(entity_id) = *selected_entity {
                if ui.button("- Delete Selected").clicked() {
                    state.show_delete_confirm = true;
                    state.entity_to_delete = Some(entity_id);
                }
            }
        });

    // Create entity dialog
    if state.show_create_dialog {
        egui::Window::new("Create Entity")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Entity name:");
                ui.text_edit_singleline(&mut state.new_entity_name);

                if let Some(parent_id) = state.new_entity_parent {
                    if let Some(parent) = scene.get_entity(parent_id) {
                        ui.label(format!("Parent: {}", parent.name));
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let is_valid = !state.new_entity_name.trim().is_empty();
                    if ui.add_enabled(is_valid, egui::Button::new("Create")).clicked() {
                        action.create_entity = Some((
                            state.new_entity_name.clone(),
                            state.new_entity_parent,
                        ));
                        state.show_create_dialog = false;
                        state.new_entity_name = "New Entity".to_string();
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_create_dialog = false;
                    }
                });
            });
    }

    // Delete confirmation dialog
    if state.show_delete_confirm {
        if let Some(entity_id) = state.entity_to_delete {
            let entity_name = scene
                .get_entity(entity_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            egui::Window::new("Delete Entity")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("Delete \"{}\"?", entity_name));

                    if let Some(entity) = scene.get_entity(entity_id) {
                        if !entity.children.is_empty() {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("This will also delete {} children", entity.children.len()),
                            );
                        }
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            action.delete_entity = Some(entity_id);
                            state.show_delete_confirm = false;
                            state.entity_to_delete = None;
                        }
                        if ui.button("Cancel").clicked() {
                            state.show_delete_confirm = false;
                            state.entity_to_delete = None;
                        }
                    });
                });
        }
    }

    action
}

fn render_entity_tree(
    ui: &mut egui::Ui,
    scene: &Scene,
    entity_id: EntityId,
    selected_entity: &mut Option<EntityId>,
    state: &mut HierarchyState,
    action: &mut HierarchyAction,
    depth: usize,
) {
    let Some(entity) = scene.get_entity(entity_id) else {
        return;
    };

    // Check if this entity or any descendants match the search filter
    let search_filter = state.search_filter.to_lowercase();
    let matches_search = search_filter.is_empty() || entity_matches_search(scene, entity_id, &search_filter);

    // Skip rendering if doesn't match search and no children match
    if !matches_search {
        return;
    }

    let is_selected = *selected_entity == Some(entity_id);
    let has_children = !entity.children.is_empty();
    let entity_name = entity.name.clone();
    let children = entity.children.clone();
    let is_expanded = state.expanded_entities.contains(&entity_id);

    // Check if entity name directly matches
    let name_matches = entity_name.to_lowercase().contains(&search_filter);

    // Check if this entity is being edited
    let is_editing = state.editing_entity == Some(entity_id);

    // Horizontal layout for indent + label
    ui.horizontal(|ui| {
        // Indentation for hierarchy depth
        ui.add_space(depth as f32 * 16.0);

        // Expand/collapse button for entities with children
        if has_children {
            let expand_icon = if is_expanded { "v" } else { ">" };
            if ui.small_button(expand_icon).clicked() {
                if is_expanded {
                    state.expanded_entities.remove(&entity_id);
                } else {
                    state.expanded_entities.insert(entity_id);
                }
            }
        } else {
            ui.add_space(20.0); // Space for alignment
        }

        if is_editing {
            // Show text input for editing
            let response = ui.text_edit_singleline(&mut state.editing_name);

            // Request focus on first frame
            if response.gained_focus() || state.editing_name.is_empty() {
                // Name was just set, focus should be requested
            }
            response.request_focus();

            // Confirm on Enter
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let new_name = state.editing_name.trim().to_string();
                if !new_name.is_empty() {
                    action.rename_entity = Some((entity_id, new_name));
                }
                state.editing_entity = None;
                state.editing_name.clear();
            }
            // Cancel on Escape
            else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.editing_entity = None;
                state.editing_name.clear();
            }
            // Confirm on click outside (lost focus without Enter)
            else if response.lost_focus() {
                let new_name = state.editing_name.trim().to_string();
                if !new_name.is_empty() {
                    action.rename_entity = Some((entity_id, new_name));
                }
                state.editing_entity = None;
                state.editing_name.clear();
            }
        } else {
            // Highlight matching entities
            let label_text = if !search_filter.is_empty() && name_matches {
                egui::RichText::new(&entity_name).color(egui::Color32::YELLOW)
            } else {
                egui::RichText::new(&entity_name)
            };

            let response = ui.selectable_label(is_selected, label_text);

            // Single click to select
            if response.clicked() {
                *selected_entity = Some(entity_id);
            }

            // Double-click to start editing
            if response.double_clicked() {
                state.editing_entity = Some(entity_id);
                state.editing_name = entity_name.clone();
            }

            // Right-click context menu
            response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    state.editing_entity = Some(entity_id);
                    state.editing_name = entity_name.clone();
                    ui.close_menu();
                }
                if ui.button("Duplicate").clicked() {
                    action.duplicate_entity = Some(entity_id);
                    ui.close_menu();
                }
                if ui.button("Create Child").clicked() {
                    state.show_create_dialog = true;
                    state.new_entity_parent = Some(entity_id);
                    state.new_entity_name = "New Entity".to_string();
                    ui.close_menu();
                }
                ui.separator();
                if has_children {
                    if ui.button("Expand All").clicked() {
                        expand_all_children(scene, entity_id, &mut state.expanded_entities);
                        ui.close_menu();
                    }
                    if ui.button("Collapse All").clicked() {
                        collapse_all_children(scene, entity_id, &mut state.expanded_entities);
                        ui.close_menu();
                    }
                    ui.separator();
                }
                if ui.button("Delete").clicked() {
                    state.show_delete_confirm = true;
                    state.entity_to_delete = Some(entity_id);
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Set Parent", |ui| {
                    if ui.button("None (root)").clicked() {
                        state.context_menu_entity = Some(entity_id);
                        ui.close_menu();
                    }
                    ui.separator();
                    // List all entities as potential parents (except self and descendants)
                    for potential_parent in scene.entities() {
                        if potential_parent.id != entity_id && !is_descendant(scene, potential_parent.id, entity_id) {
                            if ui.button(&potential_parent.name).clicked() {
                                state.context_menu_entity = Some(entity_id);
                                ui.close_menu();
                            }
                        }
                    }
                });
            });
        }
    });

    // Render children if expanded (or if searching and children match)
    if has_children && (is_expanded || !search_filter.is_empty()) {
        for child_id in children {
            render_entity_tree(ui, scene, child_id, selected_entity, state, action, depth + 1);
        }
    }
}

/// Check if potential_child is a descendant of potential_parent
fn is_descendant(scene: &Scene, entity_id: EntityId, potential_parent: EntityId) -> bool {
    if let Some(entity) = scene.get_entity(entity_id) {
        if let Some(parent_id) = entity.parent {
            if parent_id == potential_parent {
                return true;
            }
            return is_descendant(scene, parent_id, potential_parent);
        }
    }
    false
}

/// Check if an entity or any of its descendants match the search filter
fn entity_matches_search(scene: &Scene, entity_id: EntityId, search_filter: &str) -> bool {
    if let Some(entity) = scene.get_entity(entity_id) {
        // Check if entity name matches
        if entity.name.to_lowercase().contains(search_filter) {
            return true;
        }
        // Check children
        for child_id in &entity.children {
            if entity_matches_search(scene, *child_id, search_filter) {
                return true;
            }
        }
    }
    false
}

/// Count entities matching the search filter
fn count_matching_entities(scene: &Scene, search_filter: &str) -> usize {
    let filter = search_filter.to_lowercase();
    scene.entities()
        .filter(|e| e.name.to_lowercase().contains(&filter))
        .count()
}

/// Expand all parent entities of matching entities
fn expand_matching_entities(
    scene: &Scene,
    search_filter: &str,
    expanded: &mut std::collections::HashSet<EntityId>,
) {
    let filter = search_filter.to_lowercase();
    for entity in scene.entities() {
        if entity.name.to_lowercase().contains(&filter) {
            // Expand all parents of this entity
            let mut current = entity.parent;
            while let Some(parent_id) = current {
                expanded.insert(parent_id);
                current = scene.get_entity(parent_id).and_then(|e| e.parent);
            }
        }
    }
}

/// Expand all children recursively
fn expand_all_children(
    scene: &Scene,
    entity_id: EntityId,
    expanded: &mut std::collections::HashSet<EntityId>,
) {
    expanded.insert(entity_id);
    if let Some(entity) = scene.get_entity(entity_id) {
        for child_id in &entity.children {
            expand_all_children(scene, *child_id, expanded);
        }
    }
}

/// Collapse all children recursively
fn collapse_all_children(
    scene: &Scene,
    entity_id: EntityId,
    expanded: &mut std::collections::HashSet<EntityId>,
) {
    expanded.remove(&entity_id);
    if let Some(entity) = scene.get_entity(entity_id) {
        for child_id in &entity.children {
            collapse_all_children(scene, *child_id, expanded);
        }
    }
}
