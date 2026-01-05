// Hierarchy panel - shows scene graph tree

use egui::{Context, ScrollArea};
use engine_scene::{entity::EntityId, scene::Scene};

pub fn render_hierarchy_panel(
    ctx: &Context,
    scene: &Scene,
    selected_entity: &mut Option<EntityId>,
) {
    egui::SidePanel::left("hierarchy_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                // Show all root entities (entities without parents)
                for entity in scene.entities() {
                    if entity.parent.is_none() {
                        render_entity_tree(ui, scene, entity.id, selected_entity, 0);
                    }
                }
            });

            ui.separator();
            if ui.button("➕ Create Entity").clicked() {
                // TODO: Create new entity
            }
        });
}

fn render_entity_tree(
    ui: &mut egui::Ui,
    scene: &Scene,
    entity_id: EntityId,
    selected_entity: &mut Option<EntityId>,
    depth: usize,
) {
    let Some(entity) = scene.get_entity(entity_id) else {
        return;
    };

    let is_selected = *selected_entity == Some(entity_id);
    let has_children = !entity.children.is_empty();

    // Indentation for hierarchy depth
    let indent = depth as f32 * 16.0;
    ui.add_space(indent);

    // Create a selectable label with icon
    let icon = if has_children { "📁" } else { "📄" };
    let label = format!("{} {}", icon, entity.name);

    let response = ui.selectable_label(is_selected, label);
    if response.clicked() {
        *selected_entity = Some(entity_id);
    }

    // Render children
    if has_children {
        let children = entity.children.clone(); // Clone to avoid borrow issues
        for child_id in children {
            render_entity_tree(ui, scene, child_id, selected_entity, depth + 1);
        }
    }
}
