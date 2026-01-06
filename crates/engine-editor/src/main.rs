// Causality Engine - Editor

mod ui;
pub mod ipc;
mod file_ipc;

use anyhow::Result;
use engine_assets::{manager::AssetManager, mesh::Mesh, HotReloadWatcher, ReloadEvent};
use wgpu::util::DeviceExt;
use engine_physics::{Collider, PhysicsSync, PhysicsWorld, RigidBody};
use engine_render::{
    camera::Camera,
    gpu_mesh::GpuVertex,
    mesh_manager::MeshManager,
    postprocess::{Framebuffer, PostProcessPipeline},
    renderer::Renderer,
    shadow::ShadowMap,
    skybox::Skybox,
};
use engine_scene::{
    components::MeshRenderer,
    entity::EntityId,
    scene::Scene,
    transform::Transform,
};
use engine_scripting::{Script, ScriptSystem};
use glam::{Quat, Vec3};
use std::sync::Arc;
use ui::{viewport::ViewportControls, EditorUi};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct EditorApp {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState>,
    camera: Option<Camera>,
    scene: Option<Scene>,
    asset_manager: Option<AssetManager>,
    physics_world: Option<PhysicsWorld>,
    script_system: Option<ScriptSystem>,
    entity_ids: Vec<EntityId>,
    time: f32,
    ui: Option<EditorUi>,
    egui_state: Option<EguiState>,
    viewport_controls: ViewportControls,
    hot_reload: Option<HotReloadWatcher>,
    script_paths: std::collections::HashMap<EntityId, std::path::PathBuf>,
    ipc_channel: Option<ipc::IpcChannel>,
    file_ipc: Option<file_ipc::FileIpcHandler>,
}

struct EguiState {
    context: egui::Context,
    winit_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

struct WgpuState {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    renderer: Renderer,
    mesh_manager: MeshManager,
    depth_texture: wgpu::TextureView,
    skybox: Option<Skybox>,
    shadow_map: Option<ShadowMap>,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: Option<wgpu::BindGroup>,
    camera_uniform_buffer: wgpu::Buffer,
    framebuffer: Option<Framebuffer>,
    post_process_pipeline: Option<PostProcessPipeline>,
}

// Uniforms and push constants now handled by renderer
// No need to redefine here since render_mesh handles it

// Helper function to convert CPU mesh to GPU vertex format
fn convert_mesh_to_gpu(mesh: &Mesh) -> Vec<GpuVertex> {
    mesh.vertices
        .iter()
        .map(|v| GpuVertex {
            position: v.position.to_array(),
            normal: v.normal.to_array(),
            tex_coord: v.tex_coord.to_array(),
            color: v.color.unwrap_or(Vec3::ONE).to_array(),
        })
        .collect()
}

impl EditorApp {
    fn new() -> Self {
        Self {
            window: None,
            wgpu_state: None,
            camera: None,
            scene: None,
            asset_manager: None,
            physics_world: None,
            script_system: None,
            entity_ids: Vec::new(),
            time: 0.0,
            ui: None,
            egui_state: None,
            viewport_controls: ViewportControls::new(),
            hot_reload: None,
            script_paths: std::collections::HashMap::new(),
            ipc_channel: None,
            file_ipc: Some(file_ipc::FileIpcHandler::new()),
        }
    }

    fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let renderer = pollster::block_on(Renderer::new(
            &instance,
            &surface,
            size.width,
            size.height,
        ))?;

        let depth_texture = renderer.create_depth_texture(size.width, size.height);
        let camera = Camera::new(size.width, size.height);

        // Create asset and mesh managers
        let asset_manager = AssetManager::new(std::env::current_dir()?.join("assets"));
        let mut mesh_manager = MeshManager::new();

        // Create SIMPLE test scene with obvious cubes
        let mut scene = Scene::new("Simple Test Scene".to_string());

        // Red cube
        let red_cube = Mesh::cube_with_color(Vec3::new(1.0, 0.0, 0.0));
        let red_vertices = convert_mesh_to_gpu(&red_cube);
        mesh_manager.upload_mesh(&renderer.device, "red_cube".to_string(), &red_vertices, &red_cube.indices);

        // Green cube
        let green_cube = Mesh::cube_with_color(Vec3::new(0.0, 1.0, 0.0));
        let green_vertices = convert_mesh_to_gpu(&green_cube);
        mesh_manager.upload_mesh(&renderer.device, "green_cube".to_string(), &green_vertices, &green_cube.indices);

        // Blue cube
        let blue_cube = Mesh::cube_with_color(Vec3::new(0.0, 0.0, 1.0));
        let blue_vertices = convert_mesh_to_gpu(&blue_cube);
        mesh_manager.upload_mesh(&renderer.device, "blue_cube".to_string(), &blue_vertices, &blue_cube.indices);

        let mut entity_ids = Vec::new();

        // Red cube at origin
        let red_id = scene.create_entity("Red Cube".to_string());
        if let Some(entity) = scene.get_entity_mut(red_id) {
            entity.transform.position = Vec3::new(0.0, 0.0, 0.0);
            entity.transform.scale = Vec3::ONE;
            entity.add_component(MeshRenderer {
                mesh_path: "red_cube".to_string(),
                material_path: None,
            });
            log::info!("Created RED cube at {:?}", entity.transform.position);
        }
        entity_ids.push(red_id);

        // Green cube to the right
        let green_id = scene.create_entity("Green Cube".to_string());
        if let Some(entity) = scene.get_entity_mut(green_id) {
            entity.transform.position = Vec3::new(2.0, 0.0, 0.0);
            entity.transform.scale = Vec3::ONE;
            entity.add_component(MeshRenderer {
                mesh_path: "green_cube".to_string(),
                material_path: None,
            });
            log::info!("Created GREEN cube at {:?}", entity.transform.position);
        }
        entity_ids.push(green_id);

        // Blue cube above
        let blue_id = scene.create_entity("Blue Cube".to_string());
        if let Some(entity) = scene.get_entity_mut(blue_id) {
            entity.transform.position = Vec3::new(0.0, 2.0, 0.0);
            entity.transform.scale = Vec3::ONE;
            entity.add_component(MeshRenderer {
                mesh_path: "blue_cube".to_string(),
                material_path: None,
            });
            log::info!("Created BLUE cube at {:?}", entity.transform.position);
        }
        entity_ids.push(blue_id);

        // Store entity IDs for reference
        let entity_ids = entity_ids;

        // Skip the old castle scene code
        /*
        // === OLD CASTLE SCENE - REMOVED FOR TESTING ===
        // Ground plane - scaled to fit view
        let countryside_id = scene.create_entity("Countryside Ground".to_string());
        if let Some(entity) = scene.get_entity_mut(countryside_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, -0.5, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(20.0, 0.2, 20.0), // Reasonable ground size
            };
            entity.add_component(MeshRenderer {
                mesh_path: "grass_cube".to_string(),
                material_path: None,
            });
            entity.add_component(RigidBody::static_body());
            entity.add_component(Collider::box_collider(Vec3::new(10.0, 0.1, 10.0)));
        }
        entity_ids.push(countryside_id);

        // === MOAT SYSTEM ===
        // Moat water basin - square ring around castle (scaled down)
        let moat_positions = vec![
            ("Moat North", Vec3::new(0.0, -0.8, -4.5), Vec3::new(10.0, 0.8, 1.0)),
            ("Moat South", Vec3::new(0.0, -0.8, 4.5), Vec3::new(10.0, 0.8, 1.0)),
            ("Moat East", Vec3::new(4.5, -0.8, 0.0), Vec3::new(1.0, 0.8, 8.0)),
            ("Moat West", Vec3::new(-4.5, -0.8, 0.0), Vec3::new(1.0, 0.8, 8.0)),
        ];

        for (name, pos, scale) in moat_positions {
            let moat_id = scene.create_entity(name.to_string());
            if let Some(entity) = scene.get_entity_mut(moat_id) {
                entity.transform = Transform {
                    position: pos,
                    rotation: Quat::IDENTITY,
                    scale,
                };
                entity.add_component(MeshRenderer {
                    mesh_path: "water_cube".to_string(),
                    material_path: None,
                });
                entity.add_component(RigidBody::static_body());
                entity.add_component(Collider::box_collider(scale / 2.0));
            }
            entity_ids.push(moat_id);
        }

        // === CASTLE WALLS (CURTAIN WALLS) ===
        let castle_walls = vec![
            ("Castle Wall North", Vec3::new(0.0, 1.5, -3.5), Vec3::new(7.0, 3.0, 0.4)),
            ("Castle Wall South", Vec3::new(0.0, 1.5, 3.5), Vec3::new(7.0, 3.0, 0.4)),
            ("Castle Wall East", Vec3::new(3.5, 1.5, 0.0), Vec3::new(0.4, 3.0, 7.0)),
            ("Castle Wall West", Vec3::new(-3.5, 1.5, 0.0), Vec3::new(0.4, 3.0, 7.0)),
        ];

        for (name, pos, scale) in castle_walls {
            let wall_id = scene.create_entity(name.to_string());
            if let Some(entity) = scene.get_entity_mut(wall_id) {
                entity.transform = Transform {
                    position: pos,
                    rotation: Quat::IDENTITY,
                    scale,
                };
                entity.add_component(MeshRenderer {
                    mesh_path: "stone_cube".to_string(),
                    material_path: None,
                });
                entity.add_component(RigidBody::static_body());
                entity.add_component(Collider::box_collider(scale / 2.0));
            }
            entity_ids.push(wall_id);
        }

        // === CORNER TOWERS (DEFENSIVE TURRETS) ===
        let corner_towers = vec![
            ("Tower NE", Vec3::new(3.5, 2.0, -3.5)),
            ("Tower SE", Vec3::new(3.5, 2.0, 3.5)),
            ("Tower NW", Vec3::new(-3.5, 2.0, -3.5)),
            ("Tower SW", Vec3::new(-3.5, 2.0, 3.5)),
        ];

        for (name, pos) in corner_towers {
            let tower_id = scene.create_entity(name.to_string());
            if let Some(entity) = scene.get_entity_mut(tower_id) {
                entity.transform = Transform {
                    position: pos,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(1.0, 4.0, 1.0), // Defensive towers
                };
                entity.add_component(MeshRenderer {
                    mesh_path: "stone_cube".to_string(),
                    material_path: None,
                });
                entity.add_component(RigidBody::static_body());
                entity.add_component(Collider::box_collider(Vec3::new(0.5, 2.0, 0.5)));
            }
            entity_ids.push(tower_id);
        }

        // === CENTRAL KEEP (MAIN FORTRESS) ===
        let keep_id = scene.create_entity("Castle Keep".to_string());
        if let Some(entity) = scene.get_entity_mut(keep_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, 3.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.5, 6.0, 1.5), // Central tower
            };
            entity.add_component(MeshRenderer {
                mesh_path: "stone_cube".to_string(),
                material_path: None,
            });
            entity.add_component(RigidBody::static_body());
            entity.add_component(Collider::box_collider(Vec3::new(0.75, 3.0, 0.75)));
        }
        entity_ids.push(keep_id);

        // === GATEHOUSE ===
        let gatehouse_id = scene.create_entity("Gatehouse".to_string());
        if let Some(entity) = scene.get_entity_mut(gatehouse_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, 1.2, 3.8), // Front of south wall
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.5, 2.4, 0.8),
            };
            entity.add_component(MeshRenderer {
                mesh_path: "stone_cube".to_string(),
                material_path: None,
            });
            entity.add_component(RigidBody::static_body());
            entity.add_component(Collider::box_collider(Vec3::new(0.75, 1.2, 0.4)));
        }
        entity_ids.push(gatehouse_id);

        // === COURTYARD DETAILS ===
        // Courtyard ground
        let courtyard_id = scene.create_entity("Courtyard".to_string());
        if let Some(entity) = scene.get_entity_mut(courtyard_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(6.5, 0.1, 6.5),
            };
            entity.add_component(MeshRenderer {
                mesh_path: "stone_cube".to_string(),
                material_path: None,
            });
            entity.add_component(RigidBody::static_body());
            entity.add_component(Collider::box_collider(Vec3::new(3.25, 0.05, 3.25)));
        }
        entity_ids.push(courtyard_id);

        // === COUNTRYSIDE DETAILS (SCATTERED ELEMENTS) ===
        // Small hills/mounds
        let hills = vec![
            ("Hill 1", Vec3::new(8.0, 0.3, 8.0), Vec3::new(2.0, 0.6, 2.0)),
            ("Hill 2", Vec3::new(-8.0, 0.35, 7.0), Vec3::new(2.5, 0.7, 2.5)),
            ("Hill 3", Vec3::new(9.0, 0.25, -9.0), Vec3::new(1.8, 0.5, 1.8)),
        ];

        for (name, pos, scale) in hills {
            let hill_id = scene.create_entity(name.to_string());
            if let Some(entity) = scene.get_entity_mut(hill_id) {
                entity.transform = Transform {
                    position: pos,
                    rotation: Quat::IDENTITY,
                    scale,
                };
                entity.add_component(MeshRenderer {
                    mesh_path: "grass_cube".to_string(),
                    material_path: None,
                });
                entity.add_component(RigidBody::static_body());
                entity.add_component(Collider::box_collider(scale / 2.0));
            }
            entity_ids.push(hill_id);
        }

        */
        // End of commented castle scene

        // Initialize physics world
        let mut physics_world = PhysicsWorld::default(); // Default gravity is (0, -9.81, 0)
        PhysicsSync::initialize_physics(&mut physics_world, &scene)?;

        // Initialize script system
        let mut script_system = ScriptSystem::new();
        script_system.initialize(&scene)?;
        script_system.start(&mut scene)?;
        log::info!("Script system initialized");

        // Initialize egui
        let egui_context = egui::Context::default();
        let egui_winit_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None, // theme
            Some(2048), // max_texture_side
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.surface_config.format,
            egui_wgpu::RendererOptions::default(),
        );

        // Create camera uniform buffer and bind group layout for skybox
        use glam::Mat4;
        let view_proj = camera.view_projection_matrix();
        let view_proj_inverse = view_proj.inverse();

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct CameraUniforms {
            view_proj: [[f32; 4]; 4],
            view_proj_inverse: [[f32; 4]; 4],
        }

        let camera_uniforms = CameraUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            view_proj_inverse: view_proj_inverse.to_cols_array_2d(),
        };

        let camera_uniform_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
        });

        // Create skybox
        let skybox = Skybox::new(
            &renderer.device,
            renderer.surface_config.format,
            &camera_bind_group_layout,
        ).ok();

        // Initialize skybox with gradient
        if let Some(ref skybox) = skybox {
            Skybox::create_gradient_skybox(&renderer.queue, &skybox.texture);
        }

        // Create shadow map
        let shadow_map = ShadowMap::new(&renderer.device).ok();

        // Create framebuffer for post-processing
        let framebuffer = Framebuffer::new(
            &renderer.device,
            size.width,
            size.height,
            renderer.surface_config.format,
            true, // with depth
        ).ok();

        // Create post-processing pipeline
        let post_process_pipeline = PostProcessPipeline::new(
            &renderer.device,
            renderer.surface_config.format,
        ).ok();

        self.window = Some(window.clone());
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            mesh_manager,
            depth_texture,
            skybox,
            shadow_map,
            camera_bind_group_layout,
            camera_bind_group: Some(camera_bind_group),
            camera_uniform_buffer,
            framebuffer,
            post_process_pipeline,
        });
        self.camera = Some(camera);
        self.scene = Some(scene);
        self.asset_manager = Some(asset_manager);
        self.physics_world = Some(physics_world);
        self.script_system = Some(script_system);
        self.entity_ids = entity_ids;
        self.ui = Some(EditorUi::new());
        self.egui_state = Some(EguiState {
            context: egui_context,
            winit_state: egui_winit_state,
            renderer: egui_renderer,
        });

        // Initialize hot reload watcher
        let mut hot_reload = HotReloadWatcher::new()?;

        // Watch assets directory if it exists
        let assets_dir = std::env::current_dir()?.join("assets");
        if assets_dir.exists() {
            if let Err(e) = hot_reload.watch_directory(&assets_dir) {
                log::warn!("Failed to watch assets directory: {}", e);
            }
        }

        // Watch scripts directory if it exists
        let scripts_dir = std::env::current_dir()?.join("scripts");
        if scripts_dir.exists() {
            if let Err(e) = hot_reload.watch_directory(&scripts_dir) {
                log::warn!("Failed to watch scripts directory: {}", e);
            }
        }

        self.hot_reload = Some(hot_reload);
        log::info!("Hot reload system initialized");

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let (Some(wgpu_state), Some(camera)) = (&mut self.wgpu_state, &mut self.camera) {
                wgpu_state.renderer.resize(&wgpu_state.surface, new_size.width, new_size.height);
                wgpu_state.depth_texture = wgpu_state.renderer.create_depth_texture(new_size.width, new_size.height);
                camera.update_aspect(new_size.width, new_size.height);
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        let Some(wgpu_state) = &self.wgpu_state else {
            return Ok(());
        };
        let Some(camera) = &mut self.camera else {
            return Ok(());
        };
        let Some(scene) = &mut self.scene else {
            return Ok(());
        };
        let Some(physics_world) = &mut self.physics_world else {
            return Ok(());
        };
        let Some(script_system) = &mut self.script_system else {
            return Ok(());
        };
        let Some(window) = &self.window else {
            return Ok(());
        };

        // Fixed time step for physics (60fps)
        let dt = 1.0 / 60.0;

        // Process file-based IPC commands from MCP server
        if let Some(file_ipc) = &mut self.file_ipc {
            if let Err(e) = file_ipc.poll_commands(scene) {
                log::error!("File IPC error: {}", e);
            }
        }

        // Check for screenshot trigger and capture using system tool
        let screenshot_trigger = std::env::temp_dir().join("game-engine-screenshot-trigger");
        if screenshot_trigger.exists() {
            let _ = std::fs::remove_file(&screenshot_trigger);
            let screenshot_path = "/tmp/editor_screenshot.png";
            // Use import (ImageMagick) to capture the active window
            let _ = std::process::Command::new("import")
                .args(&["-window", "root", screenshot_path])
                .output();
            log::info!("Screenshot captured to {}", screenshot_path);
        }

        // Process hot reload events
        if let Some(hot_reload) = &mut self.hot_reload {
            let events = hot_reload.poll_events();
            for event in events {
                match event {
                    ReloadEvent::ScriptChanged(path) => {
                        log::info!("Reloading script: {:?}", path);
                        // Find entity with this script path
                        let mut entity_to_reload = None;
                        for (entity_id, script_path) in &self.script_paths {
                            if script_path == &path {
                                entity_to_reload = Some(*entity_id);
                                break;
                            }
                        }

                        if let Some(entity_id) = entity_to_reload {
                            // Reload the script
                            match std::fs::read_to_string(&path) {
                                Ok(source) => {
                                    if let Err(e) = script_system.reload_script(entity_id, source) {
                                        log::error!("Failed to reload script for entity {:?}: {}", entity_id, e);
                                        if let Some(ui) = &mut self.ui {
                                            ui.log_error(format!("Script reload failed: {}", e));
                                        }
                                    } else {
                                        log::info!("Successfully reloaded script for entity {:?}", entity_id);
                                        if let Some(ui) = &mut self.ui {
                                            ui.log_info(format!("Reloaded script: {}", path.display()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to read script file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                    ReloadEvent::TextureChanged(path) => {
                        log::info!("Texture changed: {:?}", path);
                        if let Some(asset_manager) = &mut self.asset_manager {
                            // Get relative path from absolute path
                            if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                let path_str = relative_path.to_string_lossy();
                                if let Err(e) = asset_manager.reload_texture(&path_str) {
                                    log::error!("Failed to reload texture {:?}: {}", path, e);
                                } else {
                                    log::info!("Successfully reloaded texture: {:?}", path);
                                    // TODO: Update GPU texture resources
                                }
                            }
                        }
                    }
                    ReloadEvent::ModelChanged(path) => {
                        log::info!("Model changed: {:?}", path);
                        if let Some(asset_manager) = &mut self.asset_manager {
                            // Get relative path from absolute path
                            if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                let path_str = relative_path.to_string_lossy();
                                if path_str.ends_with(".gltf") || path_str.ends_with(".glb") {
                                    if let Err(e) = asset_manager.reload_gltf(&path_str) {
                                        log::error!("Failed to reload model {:?}: {}", path, e);
                                    } else {
                                        log::info!("Successfully reloaded model: {:?}", path);
                                        // TODO: Re-upload to GPU and update entities
                                    }
                                }
                            }
                        }
                    }
                    ReloadEvent::AssetChanged(path) => {
                        log::info!("Generic asset changed: {:?}", path);
                        // Generic asset change - determine type by extension
                        let path_str = path.to_string_lossy();
                        if path_str.ends_with(".png") || path_str.ends_with(".jpg") ||
                           path_str.ends_with(".jpeg") || path_str.ends_with(".bmp") {
                            // Treat as texture
                            if let Some(asset_manager) = &mut self.asset_manager {
                                if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                    let rel_str = relative_path.to_string_lossy();
                                    let _ = asset_manager.reload_texture(&rel_str);
                                }
                            }
                        } else if path_str.ends_with(".gltf") || path_str.ends_with(".glb") {
                            // Treat as model
                            if let Some(asset_manager) = &mut self.asset_manager {
                                if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                    let rel_str = relative_path.to_string_lossy();
                                    let _ = asset_manager.reload_gltf(&rel_str);
                                }
                            }
                        }
                    }
                }
            }

            // Cleanup old debounce entries periodically
            hot_reload.cleanup_old_debounce_entries();
        }

        // Update camera from viewport controls
        self.viewport_controls.update_camera(camera);

        // Update scripts
        script_system.update(scene, dt)?;

        // Step physics simulation
        physics_world.step(dt);

        // Sync physics world back to scene transforms
        PhysicsSync::sync_to_scene(physics_world, scene)?;

        // Begin frame
        let (output, mut encoder, view) = wgpu_state.renderer.begin_frame(
            &wgpu_state.surface,
            &wgpu_state.depth_texture,
        )?;

        let view_proj = camera.view_projection_matrix();
        let view_proj_inverse = view_proj.inverse();

        // Update camera uniform buffer for skybox
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct CameraUniforms {
            view_proj: [[f32; 4]; 4],
            view_proj_inverse: [[f32; 4]; 4],
        }

        let camera_uniforms = CameraUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            view_proj_inverse: view_proj_inverse.to_cols_array_2d(),
        };

        wgpu_state.renderer.queue.write_buffer(
            &wgpu_state.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniforms]),
        );

        // Render shadow map (depth pass from light's perspective)
        if let Some(ref shadow_map) = wgpu_state.shadow_map {
            // Directional light coming from above and to the side
            let light_direction = glam::Vec3::new(0.5, -1.0, 0.3).normalize();

            // Calculate scene bounds (simplified - use all entities)
            let scene_center = glam::Vec3::ZERO;
            let scene_radius = 10.0; // Conservative estimate

            let light_space_matrix = ShadowMap::calculate_light_space_matrix(
                light_direction,
                scene_center,
                scene_radius,
            );

            // Shadow pass - render all meshes from light's perspective
            {
                let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shadow Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &shadow_map.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                shadow_pass.set_pipeline(&shadow_map.render_pipeline);

                // Render all meshes to shadow map
                for entity in scene.entities() {
                    if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                        if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&mesh_renderer.mesh_path) {
                            if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                                let world_matrix = scene.world_matrix(entity.id);

                                // Update shadow uniforms
                                shadow_map.update_uniforms(
                                    &wgpu_state.renderer.queue,
                                    light_space_matrix,
                                    world_matrix,
                                );

                                shadow_pass.set_bind_group(0, &shadow_map.bind_group, &[]);
                                shadow_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                                shadow_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                shadow_pass.draw_indexed(0..gpu_mesh.num_indices, 0, 0..1);
                            }
                        }
                    }
                }
            } // shadow_pass dropped here
        }

        // Render skybox first (background)
        if let Some(ref skybox) = wgpu_state.skybox {
            if let Some(ref camera_bind_group) = wgpu_state.camera_bind_group {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Skybox Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &wgpu_state.depth_texture,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(&skybox.render_pipeline);
                render_pass.set_bind_group(0, camera_bind_group, &[]);
                render_pass.set_bind_group(1, &skybox.bind_group, &[]);
                render_pass.draw(0..3, 0..1); // Fullscreen triangle
            }
        }

        // Render all entities using the old working method (one at a time with clear flag)
        let mut first_mesh = wgpu_state.skybox.is_none();
        for entity in scene.entities() {
            if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&mesh_renderer.mesh_path) {
                    if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                        let world_matrix = scene.world_matrix(entity.id);

                        wgpu_state.renderer.render_mesh(
                            &mut encoder,
                            &view,
                            &wgpu_state.depth_texture,
                            gpu_mesh,
                            view_proj,
                            world_matrix,
                            first_mesh,
                        );
                        first_mesh = false;
                    }
                }
            }
        }

        // Render egui UI
        let (paint_jobs, textures_delta, screen_descriptor) = {
            let ui = self.ui.as_mut().unwrap();
            let egui_state = self.egui_state.as_mut().unwrap();

            let raw_input = egui_state.winit_state.take_egui_input(window);
            let full_output = egui_state.context.run(raw_input, |ctx| {
                ui.render(ctx, scene);
            });

            egui_state.winit_state.handle_platform_output(
                window,
                full_output.platform_output,
            );

            let paint_jobs = egui_state.context.tessellate(
                full_output.shapes,
                full_output.pixels_per_point,
            );

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [wgpu_state.renderer.surface_config.width, wgpu_state.renderer.surface_config.height],
                pixels_per_point: window.scale_factor() as f32,
            };

            for (id, image_delta) in &full_output.textures_delta.set {
                egui_state.renderer.update_texture(
                    &wgpu_state.renderer.device,
                    &wgpu_state.renderer.queue,
                    *id,
                    image_delta,
                );
            }

            (paint_jobs, full_output.textures_delta, screen_descriptor)
        };

        // Update buffers and render - egui_state borrow is ended
        {
            let egui_state = self.egui_state.as_mut().unwrap();
            egui_state.renderer.update_buffers(
                &wgpu_state.renderer.device,
                &wgpu_state.renderer.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );
        }

        // Render egui UI
        let egui_state = self.egui_state.as_mut().unwrap();
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear - preserve 3D scene
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // egui-wgpu requires a 'static lifetime render pass (forget_lifetime consumes the pass)
            egui_state.renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        } // render_pass consumed by forget_lifetime()

        // Submit all rendering work
        wgpu_state.renderer.queue.submit(std::iter::once(encoder.finish()));

        {
            let egui_state = self.egui_state.as_mut().unwrap();
            // Free textures
            for id in &textures_delta.free {
                egui_state.renderer.free_texture(id);
            }
        }

        // Present
        output.present();

        Ok(())
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Causality Engine - Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    let window = Arc::new(window);
                    if let Err(e) = self.initialize(window) {
                        log::error!("Failed to initialize: {}", e);
                        event_loop.exit();
                    }
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let egui handle the event first
        if let (Some(egui_state), Some(window)) = (&mut self.egui_state, &self.window) {
            let response = egui_state.winit_state.on_window_event(
                window,
                &event,
            );
            if response.consumed {
                return; // Event was consumed by egui, don't process further
            }
        }

        // Handle viewport controls (camera)
        match &event {
            WindowEvent::MouseInput { state, button, .. } => {
                self.viewport_controls.handle_mouse_button(*button, *state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.viewport_controls.handle_mouse_motion(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.viewport_controls.handle_mouse_wheel(*delta);
            }
            _ => {}
        }

        // Handle other window events
        match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
                ..
            } => {
                log::info!("Closing...");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    log::error!("Render error: {}", e);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process IPC commands from MCP server
        if let Some(ipc) = &self.ipc_channel {
            if let Ok(Some(command)) = ipc.try_recv_command() {
                log::info!("Received IPC command: {:?}", command);

                if let Some(scene) = &mut self.scene {
                    let response = ipc::execute_command(command, scene);
                    if let Err(e) = ipc.send_response(response) {
                        log::error!("Failed to send IPC response: {}", e);
                    }
                } else {
                    let response = ipc::EditorResponse::Error {
                        message: "Scene not initialized".to_string(),
                    };
                    if let Err(e) = ipc.send_response(response) {
                        log::error!("Failed to send IPC response: {}", e);
                    }
                }
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("Causality Engine - Editor starting...");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
