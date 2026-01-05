// Inspector panel - shows entity properties

use egui::{Context, ScrollArea};
use engine_scene::{entity::EntityId, scene::Scene};

pub fn render_inspector_panel(
    ctx: &Context,
    scene: &mut Scene,
    selected_entity: &mut Option<EntityId>,
) {
    egui::SidePanel::right("inspector_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                if let Some(entity_id) = *selected_entity {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Entity name
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut entity.name);
                        ui.add_space(10.0);

                        // Transform component
                        ui.collapsing("Transform", |ui| {
                            ui.label("Position:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut entity.transform.position.x).speed(0.1));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut entity.transform.position.y).speed(0.1));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut entity.transform.position.z).speed(0.1));
                            });

                            ui.add_space(5.0);
                            ui.label("Scale:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut entity.transform.scale.x).speed(0.01));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut entity.transform.scale.y).speed(0.01));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut entity.transform.scale.z).speed(0.01));
                            });

                            ui.add_space(5.0);
                            ui.label("Rotation (Quaternion):");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.x).speed(0.01));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.y).speed(0.01));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.z).speed(0.01));
                                ui.label("W:");
                                ui.add(egui::DragValue::new(&mut entity.transform.rotation.w).speed(0.01));
                            });
                        });

                        ui.add_space(10.0);

                        // Component list
                        ui.label("Components:");
                        ui.label("(component inspection not yet implemented)");

                        // TODO: Display specific component types (MeshRenderer, RigidBody, etc.)
                    } else {
                        ui.label("Entity not found");
                        *selected_entity = None;
                    }
                } else {
                    ui.label("No entity selected");
                    ui.add_space(10.0);
                    ui.label("Select an entity from the Hierarchy panel to view its properties.");
                }
            });
        });
}
