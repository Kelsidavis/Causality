// Inspector panel - shows entity properties

use egui::{Context, ScrollArea};
use glam::Quat;
use engine_scene::{
    components::{
        Camera, Light, LightType, MeshRenderer, ParticleEmitter, TerrainGenerator, TerrainWater, Water,
    },
    entity::EntityId,
    scene::Scene,
};

/// Result of rendering the inspector panel - indicates what changed
#[derive(Default)]
pub struct InspectorResult {
    pub terrain_changed: bool,
    pub water_changed: bool,
    pub components_changed: bool,
}

/// State for the inspector panel including snapping settings
#[derive(Clone)]
pub struct InspectorState {
    /// Enable snapping for position
    pub snap_position: bool,
    /// Enable snapping for scale
    pub snap_scale: bool,
    /// Enable snapping for rotation
    pub snap_rotation: bool,
    /// Position snap grid size
    pub position_grid: f32,
    /// Scale snap grid size
    pub scale_grid: f32,
    /// Rotation snap angle (degrees)
    pub rotation_grid: f32,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            snap_position: false,
            snap_scale: false,
            snap_rotation: false,
            position_grid: 1.0,
            scale_grid: 0.25,
            rotation_grid: 15.0,
        }
    }
}

impl InspectorState {
    /// Snap a value to the grid
    fn snap_value(value: f32, grid: f32) -> f32 {
        if grid > 0.0 {
            (value / grid).round() * grid
        } else {
            value
        }
    }
}

pub fn render_inspector_panel(
    ctx: &Context,
    scene: &mut Scene,
    selected_entity: &mut Option<EntityId>,
    inspector_state: &mut InspectorState,
    is_locked: bool,
) -> InspectorResult {
    let mut result = InspectorResult::default();
    let mut components_to_remove: Vec<ComponentType> = Vec::new();
    let mut component_to_add: Option<ComponentType> = None;

    egui::SidePanel::right("inspector_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            // Show locked warning
            if is_locked {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 180, 100), "🔒 Entity is locked");
                });
                ui.separator();
            }

            ScrollArea::vertical().show(ui, |ui| {
                if let Some(entity_id) = *selected_entity {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Entity name
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut entity.name);
                        ui.add_space(10.0);

                        // Transform component (always present)
                        ui.collapsing("Transform", |ui| {
                            // Snapping controls
                            ui.horizontal(|ui| {
                                ui.label("Snap:");
                                ui.checkbox(&mut inspector_state.snap_position, "Pos");
                                ui.checkbox(&mut inspector_state.snap_scale, "Scale");
                                ui.checkbox(&mut inspector_state.snap_rotation, "Rot");
                            });

                            // Grid size settings (collapsible)
                            ui.collapsing("Snap Settings", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Position Grid:");
                                    ui.add(egui::DragValue::new(&mut inspector_state.position_grid)
                                        .speed(0.1)
                                        .range(0.01..=10.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Scale Grid:");
                                    ui.add(egui::DragValue::new(&mut inspector_state.scale_grid)
                                        .speed(0.05)
                                        .range(0.01..=1.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Rotation Grid:");
                                    ui.add(egui::DragValue::new(&mut inspector_state.rotation_grid)
                                        .speed(1.0)
                                        .range(1.0..=90.0)
                                        .suffix("°"));
                                });
                            });

                            ui.separator();

                            ui.label("Position:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                let mut x = entity.transform.position.x;
                                if ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed() {
                                    entity.transform.position.x = if inspector_state.snap_position {
                                        InspectorState::snap_value(x, inspector_state.position_grid)
                                    } else {
                                        x
                                    };
                                }
                                ui.label("Y:");
                                let mut y = entity.transform.position.y;
                                if ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed() {
                                    entity.transform.position.y = if inspector_state.snap_position {
                                        InspectorState::snap_value(y, inspector_state.position_grid)
                                    } else {
                                        y
                                    };
                                }
                                ui.label("Z:");
                                let mut z = entity.transform.position.z;
                                if ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed() {
                                    entity.transform.position.z = if inspector_state.snap_position {
                                        InspectorState::snap_value(z, inspector_state.position_grid)
                                    } else {
                                        z
                                    };
                                }
                            });

                            ui.add_space(5.0);
                            ui.label("Scale:");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                let mut sx = entity.transform.scale.x;
                                if ui.add(egui::DragValue::new(&mut sx).speed(0.01)).changed() {
                                    entity.transform.scale.x = if inspector_state.snap_scale {
                                        InspectorState::snap_value(sx, inspector_state.scale_grid)
                                    } else {
                                        sx
                                    };
                                }
                                ui.label("Y:");
                                let mut sy = entity.transform.scale.y;
                                if ui.add(egui::DragValue::new(&mut sy).speed(0.01)).changed() {
                                    entity.transform.scale.y = if inspector_state.snap_scale {
                                        InspectorState::snap_value(sy, inspector_state.scale_grid)
                                    } else {
                                        sy
                                    };
                                }
                                ui.label("Z:");
                                let mut sz = entity.transform.scale.z;
                                if ui.add(egui::DragValue::new(&mut sz).speed(0.01)).changed() {
                                    entity.transform.scale.z = if inspector_state.snap_scale {
                                        InspectorState::snap_value(sz, inspector_state.scale_grid)
                                    } else {
                                        sz
                                    };
                                }
                            });

                            ui.add_space(5.0);
                            ui.label("Rotation (Euler Degrees):");
                            // Convert quaternion to euler angles for easier editing
                            let (roll, pitch, yaw) = quat_to_euler(entity.transform.rotation);
                            let mut euler_x = roll.to_degrees();
                            let mut euler_y = pitch.to_degrees();
                            let mut euler_z = yaw.to_degrees();

                            let mut rotation_changed = false;
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                if ui.add(egui::DragValue::new(&mut euler_x).speed(1.0).suffix("°")).changed() {
                                    if inspector_state.snap_rotation {
                                        euler_x = InspectorState::snap_value(euler_x, inspector_state.rotation_grid);
                                    }
                                    rotation_changed = true;
                                }
                                ui.label("Y:");
                                if ui.add(egui::DragValue::new(&mut euler_y).speed(1.0).suffix("°")).changed() {
                                    if inspector_state.snap_rotation {
                                        euler_y = InspectorState::snap_value(euler_y, inspector_state.rotation_grid);
                                    }
                                    rotation_changed = true;
                                }
                                ui.label("Z:");
                                if ui.add(egui::DragValue::new(&mut euler_z).speed(1.0).suffix("°")).changed() {
                                    if inspector_state.snap_rotation {
                                        euler_z = InspectorState::snap_value(euler_z, inspector_state.rotation_grid);
                                    }
                                    rotation_changed = true;
                                }
                            });

                            if rotation_changed {
                                entity.transform.rotation = euler_to_quat(
                                    euler_x.to_radians(),
                                    euler_y.to_radians(),
                                    euler_z.to_radians(),
                                );
                            }
                        });

                        ui.add_space(10.0);

                        // Track which components exist for "Add Component" dropdown
                        let has_mesh_renderer = entity.has_component::<MeshRenderer>();
                        let has_camera = entity.has_component::<Camera>();
                        let has_light = entity.has_component::<Light>();
                        let has_water = entity.has_component::<Water>();
                        let has_terrain_water = entity.has_component::<TerrainWater>();
                        let has_terrain_gen = entity.has_component::<TerrainGenerator>();
                        let has_particle = entity.has_component::<ParticleEmitter>();

                        // MeshRenderer component
                        if let Some(mesh_renderer) = entity.get_component_mut::<MeshRenderer>() {
                            if render_component_header(ui, "MeshRenderer") {
                                components_to_remove.push(ComponentType::MeshRenderer);
                            }
                            render_mesh_renderer_ui(ui, mesh_renderer);
                            ui.add_space(5.0);
                        }

                        // Camera component
                        if let Some(camera) = entity.get_component_mut::<Camera>() {
                            if render_component_header(ui, "Camera") {
                                components_to_remove.push(ComponentType::Camera);
                            }
                            render_camera_ui(ui, camera);
                            ui.add_space(5.0);
                        }

                        // Light component
                        if let Some(light) = entity.get_component_mut::<Light>() {
                            if render_component_header(ui, "Light") {
                                components_to_remove.push(ComponentType::Light);
                            }
                            render_light_ui(ui, light);
                            ui.add_space(5.0);
                        }

                        // TerrainGenerator component
                        if let Some(terrain_gen) = entity.get_component_mut::<TerrainGenerator>() {
                            if render_component_header(ui, "Terrain Generator") {
                                components_to_remove.push(ComponentType::TerrainGenerator);
                            }
                            result.terrain_changed |= render_terrain_generator_ui(ui, terrain_gen);
                            ui.add_space(5.0);
                        }

                        // TerrainWater component
                        if let Some(terrain_water) = entity.get_component_mut::<TerrainWater>() {
                            if render_component_header(ui, "Terrain Water") {
                                components_to_remove.push(ComponentType::TerrainWater);
                            }
                            result.water_changed |= render_terrain_water_ui(ui, terrain_water);
                            ui.add_space(5.0);
                        }

                        // Water component
                        if let Some(water) = entity.get_component_mut::<Water>() {
                            if render_component_header(ui, "Water") {
                                components_to_remove.push(ComponentType::Water);
                            }
                            render_water_ui(ui, water);
                            ui.add_space(5.0);
                        }

                        // ParticleEmitter component
                        if let Some(particle) = entity.get_component_mut::<ParticleEmitter>() {
                            if render_component_header(ui, "Particle Emitter") {
                                components_to_remove.push(ComponentType::ParticleEmitter);
                            }
                            render_particle_emitter_ui(ui, particle);
                            ui.add_space(5.0);
                        }

                        // Add Component dropdown
                        ui.separator();
                        ui.add_space(5.0);
                        egui::ComboBox::from_label("Add Component")
                            .selected_text("Select...")
                            .show_ui(ui, |ui| {
                                if !has_mesh_renderer && ui.selectable_label(false, "MeshRenderer").clicked() {
                                    component_to_add = Some(ComponentType::MeshRenderer);
                                }
                                if !has_camera && ui.selectable_label(false, "Camera").clicked() {
                                    component_to_add = Some(ComponentType::Camera);
                                }
                                if !has_light && ui.selectable_label(false, "Light").clicked() {
                                    component_to_add = Some(ComponentType::Light);
                                }
                                if !has_water && ui.selectable_label(false, "Water").clicked() {
                                    component_to_add = Some(ComponentType::Water);
                                }
                                if !has_terrain_water && ui.selectable_label(false, "TerrainWater").clicked() {
                                    component_to_add = Some(ComponentType::TerrainWater);
                                }
                                if !has_terrain_gen && ui.selectable_label(false, "TerrainGenerator").clicked() {
                                    component_to_add = Some(ComponentType::TerrainGenerator);
                                }
                                if !has_particle && ui.selectable_label(false, "ParticleEmitter").clicked() {
                                    component_to_add = Some(ComponentType::ParticleEmitter);
                                }
                            });
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

    // Process component removals (after UI rendering to avoid borrow issues)
    if let Some(entity_id) = *selected_entity {
        if let Some(entity) = scene.get_entity_mut(entity_id) {
            for comp_type in components_to_remove {
                match comp_type {
                    ComponentType::MeshRenderer => { entity.remove_component::<MeshRenderer>(); }
                    ComponentType::Camera => { entity.remove_component::<Camera>(); }
                    ComponentType::Light => { entity.remove_component::<Light>(); }
                    ComponentType::Water => { entity.remove_component::<Water>(); }
                    ComponentType::TerrainWater => { entity.remove_component::<TerrainWater>(); }
                    ComponentType::TerrainGenerator => { entity.remove_component::<TerrainGenerator>(); }
                    ComponentType::ParticleEmitter => { entity.remove_component::<ParticleEmitter>(); }
                }
                result.components_changed = true;
            }

            // Process component additions
            if let Some(comp_type) = component_to_add {
                match comp_type {
                    ComponentType::MeshRenderer => {
                        entity.add_component(MeshRenderer::new("cube".to_string()));
                    }
                    ComponentType::Camera => {
                        entity.add_component(Camera::default());
                    }
                    ComponentType::Light => {
                        entity.add_component(Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0], 1.0));
                    }
                    ComponentType::Water => {
                        entity.add_component(Water::default());
                    }
                    ComponentType::TerrainWater => {
                        entity.add_component(TerrainWater::default());
                    }
                    ComponentType::TerrainGenerator => {
                        entity.add_component(TerrainGenerator::default());
                    }
                    ComponentType::ParticleEmitter => {
                        entity.add_component(ParticleEmitter::default());
                    }
                }
                result.components_changed = true;
            }
        }
    }

    result
}

#[derive(Clone, Copy)]
enum ComponentType {
    MeshRenderer,
    Camera,
    Light,
    Water,
    TerrainWater,
    TerrainGenerator,
    ParticleEmitter,
}

/// Render a component header with remove button. Returns true if remove was clicked.
fn render_component_header(ui: &mut egui::Ui, name: &str) -> bool {
    let mut remove = false;
    ui.horizontal(|ui| {
        ui.strong(name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("X").clicked() {
                remove = true;
            }
        });
    });
    remove
}

/// Render UI for MeshRenderer component
fn render_mesh_renderer_ui(ui: &mut egui::Ui, mesh: &mut MeshRenderer) {
    ui.horizontal(|ui| {
        ui.label("Mesh:");
        ui.text_edit_singleline(&mut mesh.mesh_path);
    });
    ui.horizontal(|ui| {
        ui.label("Material:");
        let mut mat_path = mesh.material_path.clone().unwrap_or_default();
        if ui.text_edit_singleline(&mut mat_path).changed() {
            mesh.material_path = if mat_path.is_empty() { None } else { Some(mat_path) };
        }
    });
}

/// Render UI for Camera component
fn render_camera_ui(ui: &mut egui::Ui, camera: &mut Camera) {
    ui.horizontal(|ui| {
        ui.label("FOV (deg):");
        let mut fov_deg = camera.fov.to_degrees();
        if ui.add(egui::DragValue::new(&mut fov_deg).speed(1.0).range(10.0..=120.0)).changed() {
            camera.fov = fov_deg.to_radians();
        }
    });
    ui.horizontal(|ui| {
        ui.label("Near:");
        ui.add(egui::DragValue::new(&mut camera.near).speed(0.01).range(0.001..=10.0));
    });
    ui.horizontal(|ui| {
        ui.label("Far:");
        ui.add(egui::DragValue::new(&mut camera.far).speed(1.0).range(10.0..=10000.0));
    });
    ui.checkbox(&mut camera.is_active, "Active");
}

/// Render UI for Light component
fn render_light_ui(ui: &mut egui::Ui, light: &mut Light) {
    // Light type selector
    let current_type = match &light.light_type {
        LightType::Directional { .. } => "Directional",
        LightType::Point { .. } => "Point",
        LightType::Spot { .. } => "Spot",
    };

    egui::ComboBox::from_label("Type")
        .selected_text(current_type)
        .show_ui(ui, |ui| {
            if ui.selectable_label(matches!(light.light_type, LightType::Directional { .. }), "Directional").clicked() {
                light.light_type = LightType::Directional { direction: [0.0, -1.0, 0.0] };
            }
            if ui.selectable_label(matches!(light.light_type, LightType::Point { .. }), "Point").clicked() {
                light.light_type = LightType::Point { range: 10.0, intensity: 1.0 };
            }
            if ui.selectable_label(matches!(light.light_type, LightType::Spot { .. }), "Spot").clicked() {
                light.light_type = LightType::Spot { direction: [0.0, -1.0, 0.0], angle: 45.0, range: 10.0 };
            }
        });

    // Type-specific properties
    match &mut light.light_type {
        LightType::Directional { direction } => {
            ui.horizontal(|ui| {
                ui.label("Dir X:");
                ui.add(egui::DragValue::new(&mut direction[0]).speed(0.01).range(-1.0..=1.0));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut direction[1]).speed(0.01).range(-1.0..=1.0));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut direction[2]).speed(0.01).range(-1.0..=1.0));
            });
        }
        LightType::Point { range, intensity } => {
            ui.horizontal(|ui| {
                ui.label("Range:");
                ui.add(egui::DragValue::new(range).speed(0.5).range(0.1..=100.0));
            });
            ui.horizontal(|ui| {
                ui.label("Intensity:");
                ui.add(egui::DragValue::new(intensity).speed(0.1).range(0.0..=10.0));
            });
        }
        LightType::Spot { direction, angle, range } => {
            ui.horizontal(|ui| {
                ui.label("Dir X:");
                ui.add(egui::DragValue::new(&mut direction[0]).speed(0.01).range(-1.0..=1.0));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut direction[1]).speed(0.01).range(-1.0..=1.0));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut direction[2]).speed(0.01).range(-1.0..=1.0));
            });
            ui.horizontal(|ui| {
                ui.label("Angle:");
                ui.add(egui::DragValue::new(angle).speed(1.0).range(1.0..=90.0));
            });
            ui.horizontal(|ui| {
                ui.label("Range:");
                ui.add(egui::DragValue::new(range).speed(0.5).range(0.1..=100.0));
            });
        }
    }

    // Common properties
    ui.horizontal(|ui| {
        ui.label("Color:");
        ui.color_edit_button_rgb(&mut light.color);
    });
    ui.horizontal(|ui| {
        ui.label("Intensity:");
        ui.add(egui::DragValue::new(&mut light.intensity).speed(0.1).range(0.0..=10.0));
    });
}

/// Render UI for TerrainGenerator component, returns true if changed
fn render_terrain_generator_ui(ui: &mut egui::Ui, terrain: &mut TerrainGenerator) -> bool {
    let mut changed = false;

    // Grid dimensions
    ui.label("Grid Size:");
    ui.horizontal(|ui| {
        ui.label("Width:");
        let mut width = terrain.width as i32;
        if ui.add(egui::DragValue::new(&mut width).range(8..=256)).changed() {
            terrain.width = width.max(8) as usize;
            changed = true;
        }
        ui.label("Depth:");
        let mut depth = terrain.depth as i32;
        if ui.add(egui::DragValue::new(&mut depth).range(8..=256)).changed() {
            terrain.depth = depth.max(8) as usize;
            changed = true;
        }
    });

    ui.add_space(5.0);

    // World scale
    ui.horizontal(|ui| {
        ui.label("Scale:");
        changed |= ui.add(egui::DragValue::new(&mut terrain.scale).speed(0.5).range(1.0..=500.0)).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Height Scale:");
        changed |= ui.add(egui::DragValue::new(&mut terrain.height_scale).speed(0.1).range(0.1..=50.0)).changed();
    });

    ui.add_space(5.0);

    // Noise parameters
    ui.horizontal(|ui| {
        ui.label("Seed:");
        changed |= ui.add(egui::DragValue::new(&mut terrain.seed)).changed();
    });

    ui.add_space(5.0);
    ui.separator();

    // Moat settings
    changed |= ui.checkbox(&mut terrain.moat_enabled, "Enable Moat").changed();

    if terrain.moat_enabled {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.label("Inner Radius:");
            changed |= ui.add(egui::DragValue::new(&mut terrain.moat_inner_radius).speed(0.01).range(0.0..=1.0)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Outer Radius:");
            changed |= ui.add(egui::DragValue::new(&mut terrain.moat_outer_radius).speed(0.01).range(0.0..=1.0)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("Moat Depth:");
            changed |= ui.add(egui::DragValue::new(&mut terrain.moat_depth).speed(0.01).range(0.0..=2.0)).changed();
        });
    }

    ui.add_space(5.0);

    // Regenerate button
    if ui.button("Regenerate Terrain").clicked() {
        changed = true;
    }

    changed
}

/// Render UI for TerrainWater component, returns true if changed
fn render_terrain_water_ui(ui: &mut egui::Ui, water: &mut TerrainWater) -> bool {
    let mut changed = false;

    // Water level
    ui.horizontal(|ui| {
        ui.label("Ground Water Level:");
        changed |= ui.add(egui::DragValue::new(&mut water.ground_water_level).speed(0.1)).changed();
    });

    ui.add_space(3.0);

    // Filtering parameters
    ui.horizontal(|ui| {
        ui.label("Min Water Depth:");
        changed |= ui.add(egui::DragValue::new(&mut water.min_water_depth).speed(0.01).range(0.0..=1.0)).changed();
    });
    ui.horizontal(|ui| {
        ui.label("Min Water Area:");
        let mut area = water.min_water_area as i32;
        if ui.add(egui::DragValue::new(&mut area).range(1..=100)).changed() {
            water.min_water_area = area.max(1) as usize;
            changed = true;
        }
    });

    ui.add_space(5.0);
    ui.separator();
    ui.label("Rendering:");
    ui.add_space(3.0);

    // Rendering properties
    ui.horizontal(|ui| {
        ui.label("Wave Speed:");
        ui.add(egui::DragValue::new(&mut water.wave_speed).speed(0.01).range(0.0..=5.0));
    });
    ui.horizontal(|ui| {
        ui.label("Wave Amplitude:");
        ui.add(egui::DragValue::new(&mut water.wave_amplitude).speed(0.001).range(0.0..=0.5));
    });
    ui.horizontal(|ui| {
        ui.label("Transparency:");
        ui.add(egui::DragValue::new(&mut water.transparency).speed(0.01).range(0.0..=1.0));
    });

    // Color editor
    ui.horizontal(|ui| {
        ui.label("Color:");
        let mut color = water.color;
        if ui.color_edit_button_rgb(&mut color).changed() {
            water.color = color;
        }
    });

    ui.add_space(5.0);

    // Regenerate button
    if ui.button("Regenerate Water").clicked() {
        changed = true;
    }

    changed
}

/// Render UI for Water component (non-terrain water)
fn render_water_ui(ui: &mut egui::Ui, water: &mut Water) {
    ui.horizontal(|ui| {
        ui.label("Mesh:");
        ui.text_edit_singleline(&mut water.mesh_path);
    });
    ui.horizontal(|ui| {
        ui.label("Wave Speed:");
        ui.add(egui::DragValue::new(&mut water.wave_speed).speed(0.01).range(0.0..=5.0));
    });
    ui.horizontal(|ui| {
        ui.label("Wave Frequency:");
        ui.add(egui::DragValue::new(&mut water.wave_frequency).speed(0.1).range(0.1..=10.0));
    });
    ui.horizontal(|ui| {
        ui.label("Wave Amplitude:");
        ui.add(egui::DragValue::new(&mut water.wave_amplitude).speed(0.01).range(0.0..=1.0));
    });
    ui.horizontal(|ui| {
        ui.label("Transparency:");
        ui.add(egui::DragValue::new(&mut water.transparency).speed(0.01).range(0.0..=1.0));
    });

    // Color editor
    ui.horizontal(|ui| {
        ui.label("Color:");
        ui.color_edit_button_rgb(&mut water.color);
    });

    // Flow
    ui.add_space(5.0);
    ui.label("Flow:");
    ui.horizontal(|ui| {
        ui.label("Direction X:");
        ui.add(egui::DragValue::new(&mut water.flow_direction[0]).speed(0.01).range(-1.0..=1.0));
        ui.label("Z:");
        ui.add(egui::DragValue::new(&mut water.flow_direction[1]).speed(0.01).range(-1.0..=1.0));
    });
    ui.horizontal(|ui| {
        ui.label("Flow Speed:");
        ui.add(egui::DragValue::new(&mut water.flow_speed).speed(0.1).range(0.0..=10.0));
    });
}

/// Render UI for ParticleEmitter component
fn render_particle_emitter_ui(ui: &mut egui::Ui, particle: &mut ParticleEmitter) {
    ui.checkbox(&mut particle.enabled, "Enabled");

    ui.horizontal(|ui| {
        ui.label("Max Particles:");
        let mut max = particle.max_particles as i32;
        if ui.add(egui::DragValue::new(&mut max).range(1..=100000)).changed() {
            particle.max_particles = max.max(1) as u32;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Rate:");
        ui.add(egui::DragValue::new(&mut particle.rate).speed(0.5).range(0.1..=1000.0));
    });

    ui.horizontal(|ui| {
        ui.label("Lifetime:");
        ui.add(egui::DragValue::new(&mut particle.lifetime).speed(0.1).range(0.1..=60.0));
    });

    ui.horizontal(|ui| {
        ui.label("Size:");
        ui.add(egui::DragValue::new(&mut particle.initial_size).speed(0.01).range(0.01..=10.0));
    });

    ui.add_space(5.0);
    ui.label("Velocity:");
    ui.horizontal(|ui| {
        ui.label("X:");
        ui.add(egui::DragValue::new(&mut particle.initial_velocity[0]).speed(0.1));
        ui.label("Y:");
        ui.add(egui::DragValue::new(&mut particle.initial_velocity[1]).speed(0.1));
        ui.label("Z:");
        ui.add(egui::DragValue::new(&mut particle.initial_velocity[2]).speed(0.1));
    });

    ui.horizontal(|ui| {
        ui.label("Randomness:");
        ui.add(egui::DragValue::new(&mut particle.velocity_randomness).speed(0.01).range(0.0..=1.0));
    });

    ui.add_space(5.0);
    ui.label("Gravity:");
    ui.horizontal(|ui| {
        ui.label("X:");
        ui.add(egui::DragValue::new(&mut particle.gravity[0]).speed(0.1));
        ui.label("Y:");
        ui.add(egui::DragValue::new(&mut particle.gravity[1]).speed(0.1));
        ui.label("Z:");
        ui.add(egui::DragValue::new(&mut particle.gravity[2]).speed(0.1));
    });

    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("Color:");
        let mut color = [particle.initial_color[0], particle.initial_color[1], particle.initial_color[2]];
        if ui.color_edit_button_rgb(&mut color).changed() {
            particle.initial_color[0] = color[0];
            particle.initial_color[1] = color[1];
            particle.initial_color[2] = color[2];
        }
    });
    ui.horizontal(|ui| {
        ui.label("Alpha:");
        ui.add(egui::DragValue::new(&mut particle.initial_color[3]).speed(0.01).range(0.0..=1.0));
    });
}

/// Convert quaternion to euler angles (roll, pitch, yaw) in radians
fn quat_to_euler(q: Quat) -> (f32, f32, f32) {
    // Roll (X axis rotation)
    let sinr_cosp = 2.0 * (q.w * q.x + q.y * q.z);
    let cosr_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
    let roll = sinr_cosp.atan2(cosr_cosp);

    // Pitch (Y axis rotation)
    let sinp = 2.0 * (q.w * q.y - q.z * q.x);
    let pitch = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp)
    } else {
        sinp.asin()
    };

    // Yaw (Z axis rotation)
    let siny_cosp = 2.0 * (q.w * q.z + q.x * q.y);
    let cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
    let yaw = siny_cosp.atan2(cosy_cosp);

    (roll, pitch, yaw)
}

/// Convert euler angles (roll, pitch, yaw) in radians to quaternion
fn euler_to_quat(roll: f32, pitch: f32, yaw: f32) -> Quat {
    Quat::from_euler(glam::EulerRot::XYZ, roll, pitch, yaw)
}
