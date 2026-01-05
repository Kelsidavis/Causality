// Game engine editor - Phase 3: Physics simulation

use anyhow::Result;
use engine_assets::{manager::AssetManager, mesh::Mesh};
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
use glam::{Quat, Vec3};
use std::sync::Arc;
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
    entity_ids: Vec<EntityId>,
    time: f32,
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
            entity_ids: Vec::new(),
            time: 0.0,
        }
    }

    fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
        }

        // Store entity IDs for reference
        let entity_ids = vec![cube_id, plane_id, small_cube_id];

        // Initialize physics world
        let mut physics_world = PhysicsWorld::default(); // Default gravity is (0, -9.81, 0)
        PhysicsSync::initialize_physics(&mut physics_world, &scene)?;

        self.window = Some(window);
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
        self.entity_ids = entity_ids;

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
        let Some(camera) = &self.camera else {
            return Ok(());
        };
        let Some(scene) = &mut self.scene else {
            return Ok(());
        };
        let Some(physics_world) = &mut self.physics_world else {
            return Ok(());
        };

        // Fixed time step for physics (60fps)
        let dt = 1.0 / 60.0;

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

        // End frame
        wgpu_state.renderer.end_frame(encoder, output);

        Ok(())
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Game Engine Editor - Phase 3: Physics Simulation")
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
