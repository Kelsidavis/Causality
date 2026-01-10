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
}

/// State for hierarchy UI (stored in EditorUi)
pub struct HierarchyState {
    pub show_create_dialog: bool,
    pub new_entity_name: String,
    pub new_entity_parent: Option<EntityId>,
    pub show_delete_confirm: bool,
    pub entity_to_delete: Option<EntityId>,
    pub context_menu_entity: Option<EntityId>,
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

    let is_selected = *selected_entity == Some(entity_id);
    let has_children = !entity.children.is_empty();
    let entity_name = entity.name.clone();
    let children = entity.children.clone();

    // Horizontal layout for indent + label
    ui.horizontal(|ui| {
        // Indentation for hierarchy depth
        ui.add_space(depth as f32 * 16.0);

        // Create a selectable label with icon
        let icon = if has_children { ">" } else { "-" };
        let label = format!("{} {}", icon, entity_name);

        let response = ui.selectable_label(is_selected, label);
        if response.clicked() {
            *selected_entity = Some(entity_id);
        }

        // Right-click context menu
        response.context_menu(|ui| {
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
    });

    // Render children
    if has_children {
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
