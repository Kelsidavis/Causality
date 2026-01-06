// Game engine editor - Phase 7: Hot Reload

mod ui;
pub mod ipc;

use anyhow::Result;
use engine_assets::{manager::AssetManager, mesh::Mesh, HotReloadWatcher, ReloadEvent};
use engine_physics::{Collider, PhysicsSync, PhysicsWorld, RigidBody};
use engine_render::{
    camera::Camera,
    gpu_mesh::GpuVertex,
    mesh_manager::MeshManager,
    renderer::Renderer,
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
}

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

        // Create demo scene
        let mut scene = Scene::new("Demo Scene".to_string());

        // Entity 1: Dynamic cube that will fall
        let cube_mesh = Mesh::cube();
        let cube_vertices = convert_mesh_to_gpu(&cube_mesh);
        mesh_manager.upload_mesh(&renderer.device, "cube".to_string(), &cube_vertices, &cube_mesh.indices);

        let cube_id = scene.create_entity("Falling Cube".to_string());
        if let Some(entity) = scene.get_entity_mut(cube_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, 5.0, 0.0), // Start high in the air
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            };
            entity.add_component(MeshRenderer {
                mesh_path: "cube".to_string(),
                material_path: None,
            });
            // Add physics - dynamic rigid body with box collider
            entity.add_component(RigidBody::dynamic(1.0));
            entity.add_component(Collider::box_collider(Vec3::splat(0.5))); // Half extents = 0.5 (full size 1.0)

            // Add script - slow rotation while falling
            entity.add_component(Script::new(r#"
fn update(ctx) {
    // Rotate slowly around Y axis
    let rotation_speed = 1.0;
    let angle = ctx.dt * rotation_speed;
    let rotation_delta = quat_from_rotation_y(angle);
    ctx.rotation = ctx.rotation * rotation_delta;
    ctx
}
"#.to_string()));
        }

        // Entity 2: Static ground plane
        let plane_mesh = Mesh::plane(10.0);
        let plane_vertices = convert_mesh_to_gpu(&plane_mesh);
        mesh_manager.upload_mesh(&renderer.device, "plane".to_string(), &plane_vertices, &plane_mesh.indices);

        let plane_id = scene.create_entity("Ground Plane".to_string());
        if let Some(entity) = scene.get_entity_mut(plane_id) {
            entity.transform = Transform {
                position: Vec3::new(0.0, 0.0, 0.0), // Ground at y=0
                rotation: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0),
            };
            entity.add_component(MeshRenderer {
                mesh_path: "plane".to_string(),
                material_path: None,
            });
            // Add physics - static rigid body with box collider (thin ground)
            entity.add_component(RigidBody::static_body());
            entity.add_component(Collider::box_collider(Vec3::new(5.0, 0.1, 5.0))); // Thin platform
        }

        // Entity 3: Small dynamic cube that will also fall
        let small_cube_mesh = Mesh::cube();
        let small_cube_vertices = convert_mesh_to_gpu(&small_cube_mesh);
        mesh_manager.upload_mesh(&renderer.device, "small_cube".to_string(), &small_cube_vertices, &small_cube_mesh.indices);

        let small_cube_id = scene.create_entity("Falling Small Cube".to_string());
        if let Some(entity) = scene.get_entity_mut(small_cube_id) {
            entity.transform = Transform {
                position: Vec3::new(2.0, 8.0, 0.0), // Start higher and offset
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.5), // Make it smaller
            };
            entity.add_component(MeshRenderer {
                mesh_path: "small_cube".to_string(),
                material_path: None,
            });
            // Add physics - dynamic rigid body with smaller box collider
            entity.add_component(RigidBody::dynamic(0.5)); // Lighter mass
            entity.add_component(Collider::box_collider(Vec3::splat(0.25)).with_restitution(0.3)); // Half of 0.5 scale, bouncy

            // Add script - spinning on multiple axes
            entity.add_component(Script::new(r#"
fn update(ctx) {
    // Rotate on multiple axes for interesting motion
    let spin_speed = 2.0;
    let angle = ctx.dt * spin_speed;
    let rot_x = quat_from_rotation_x(angle * 0.7);
    let rot_y = quat_from_rotation_y(angle);
    let rot_z = quat_from_rotation_z(angle * 0.5);
    ctx.rotation = ctx.rotation * rot_x * rot_y * rot_z;
    ctx
}
"#.to_string()));
        }

        // Store entity IDs for reference
        let entity_ids = vec![cube_id, plane_id, small_cube_id];

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

        self.window = Some(window.clone());
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            mesh_manager,
            depth_texture,
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

        // Render all entities with MeshRenderer component
        let mut first_mesh = true;
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
                            first_mesh, // Clear on first mesh only
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
                .with_title("Game Engine Editor - Phase 7: Hot Reload")
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
    log::info!("Game Engine Editor starting...");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
