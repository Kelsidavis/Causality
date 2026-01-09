// Causality Engine - Editor

mod ui;
pub mod ipc;
mod file_ipc;

use anyhow::Result;
use clap::Parser;
use engine_assets::{manager::{AssetHandle, AssetManager}, material::Material, mesh::Mesh, texture::Texture, HotReloadWatcher, ReloadEvent};
use wgpu::util::DeviceExt;
use engine_audio::AudioSystem;
use engine_physics::{Collider, PhysicsSync, PhysicsWorld, RigidBody};
use engine_render::{
    camera::Camera,
    gpu_mesh::GpuVertex,
    material_manager::MaterialManager,
    mesh_manager::MeshManager,
    particle_renderer::ParticleRenderer,
    postprocess::{Framebuffer, PostProcessPipeline},
    renderer::Renderer,
    shadow::ShadowMap,
    skybox::Skybox,
    texture_manager::TextureManager,
};
use engine_scene::{
    components::{AudioListener, AudioSource, MeshRenderer, ParticleEmitter},
    entity::EntityId,
    scene::Scene,
    transform::Transform,
};
use engine_scripting::{AudioCommand, AudioCommandQueue, Script, ScriptSystem};
use glam::{Quat, Vec3, Vec4};
use std::sync::{Arc, Mutex};
use ui::{viewport::ViewportControls, EditorUi};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

/// Causality Engine Editor
#[derive(Parser, Debug)]
#[command(name = "Causality Engine Editor")]
#[command(about = "3D game engine editor with visual editing tools", long_about = None)]
struct Args {
    /// Scene file to load (.ron format)
    #[arg(short, long)]
    scene: Option<String>,
}

struct EditorApp {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState>,
    camera: Option<Camera>,
    scene: Option<Scene>,
    asset_manager: Option<AssetManager>,
    physics_world: Option<PhysicsWorld>,
    script_system: Option<ScriptSystem>,
    audio_system: Option<AudioSystem>,
    audio_command_queue: AudioCommandQueue,
    entity_ids: Vec<EntityId>,
    time: f32,
    ui: Option<EditorUi>,
    egui_state: Option<EguiState>,
    viewport_controls: ViewportControls,
    hot_reload: Option<HotReloadWatcher>,
    script_paths: std::collections::HashMap<EntityId, std::path::PathBuf>,
    ipc_channel: Option<ipc::IpcChannel>,
    file_ipc: Option<file_ipc::FileIpcHandler>,
    scene_file_path: Option<String>,
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
    texture_manager: TextureManager,
    material_manager: MaterialManager,
    depth_texture: wgpu::TextureView,
    skybox: Option<Skybox>,
    shadow_map: Option<ShadowMap>,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: Option<wgpu::BindGroup>,
    camera_uniform_buffer: wgpu::Buffer,
    framebuffer: Option<Framebuffer>,
    post_process_pipeline: Option<PostProcessPipeline>,
    particle_renderer: Option<ParticleRenderer>,
    particle_systems: std::collections::HashMap<EntityId, engine_particles::ParticleSystem>,
    particle_compute_pipelines: std::collections::HashMap<EntityId, engine_particles::ParticleComputePipeline>,
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
            // Default tangent: X-axis with positive handedness
            tangent: v.tangent.unwrap_or(Vec4::new(1.0, 0.0, 0.0, 1.0)).to_array(),
            // Default bitangent: Z-axis
            bitangent: v.bitangent.unwrap_or(Vec3::Z).to_array(),
            _padding: [0.0, 0.0],
        })
        .collect()
}

impl EditorApp {
    fn new(scene_file_path: Option<String>) -> Self {
        Self {
            window: None,
            wgpu_state: None,
            camera: None,
            scene: None,
            asset_manager: None,
            physics_world: None,
            script_system: None,
            audio_system: None,
            audio_command_queue: Arc::new(Mutex::new(Vec::new())),
            entity_ids: Vec::new(),
            time: 0.0,
            ui: None,
            egui_state: None,
            viewport_controls: ViewportControls::new(),
            hot_reload: None,
            script_paths: std::collections::HashMap::new(),
            ipc_channel: None,
            file_ipc: Some(file_ipc::FileIpcHandler::new()),
            scene_file_path,
        }
    }

    fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        // Create renderer
        let renderer = pollster::block_on(Renderer::new(
            &instance,
            &surface,
            size.width,
            size.height,
        ))?;

        // Create texture manager with the renderer's device
        let mut texture_manager = TextureManager::new(&renderer.device, &renderer.queue);

        // Create material manager
        let material_manager = MaterialManager::new(&renderer.device, &texture_manager);

        let depth_texture = renderer.create_depth_texture(size.width, size.height);
        let camera = Camera::new(size.width, size.height);

        // Create asset and mesh managers
        let asset_manager = AssetManager::new(std::env::current_dir()?.join("assets"));
        let mut mesh_manager = MeshManager::new();

        // Load scene from file or create empty scene
        let mut scene = if let Some(ref scene_path) = self.scene_file_path {
            log::info!("Loading scene from: {}", scene_path);
            match Scene::load_from_file(scene_path) {
                Ok(loaded_scene) => {
                    log::info!("Loaded scene '{}' with {} entities",
                        loaded_scene.name,
                        loaded_scene.entity_count()
                    );
                    loaded_scene
                }
                Err(e) => {
                    log::error!("Failed to load scene from {}: {}", scene_path, e);
                    log::info!("Creating empty scene instead");
                    Scene::new("Empty Scene".to_string())
                }
            }
        } else {
            log::info!("No scene file specified, creating empty scene");
            Scene::new("Empty Scene".to_string())
        };

        // Load sample textures from assets folder
        log::info!("Loading sample textures...");

        // Load stone texture for castle walls (1024x1024 high-res with variation)
        if let Ok(stone_tex) = Texture::from_file("assets/textures/stone_bricks.png") {
            texture_manager.upload_texture(&renderer.device, &renderer.queue, "stone".to_string(), &stone_tex);
            log::info!("Loaded stone texture: {}x{}", stone_tex.width, stone_tex.height);
        } else {
            log::error!("Failed to load stone texture from assets/textures/stone_bricks.png");
        }

        // Load grass texture for terrain
        if let Ok(grass_tex) = Texture::from_file("assets/textures/grass.png") {
            texture_manager.upload_texture(&renderer.device, &renderer.queue, "grass".to_string(), &grass_tex);
            log::info!("Loaded grass texture: {}x{}", grass_tex.width, grass_tex.height);
        } else {
            log::error!("Failed to load grass texture from assets/textures/grass.png");
        }

        // Load water texture for moat
        if let Ok(water_tex) = Texture::from_file("assets/textures/water.png") {
            texture_manager.upload_texture(&renderer.device, &renderer.queue, "water".to_string(), &water_tex);
            log::info!("Loaded water texture: {}x{}", water_tex.width, water_tex.height);
        } else {
            log::error!("Failed to load water texture from assets/textures/water.png");
        }

        // Create cube meshes with white color (texture will provide color)
        let white = Vec3::ONE; // White tint allows texture to show through

        // Stone cube for castle structures
        let mut stone_cube = Mesh::cube_with_color(white);
        stone_cube.calculate_tangents();
        let stone_vertices = convert_mesh_to_gpu(&stone_cube);
        mesh_manager.upload_mesh(&renderer.device, "stone_cube".to_string(), &stone_vertices, &stone_cube.indices);

        // Grass cube for terrain
        let mut grass_cube = Mesh::cube_with_color(white);
        grass_cube.calculate_tangents();
        let grass_vertices = convert_mesh_to_gpu(&grass_cube);
        mesh_manager.upload_mesh(&renderer.device, "grass_cube".to_string(), &grass_vertices, &grass_cube.indices);

        // Water cube for moat
        let mut water_cube = Mesh::cube_with_color(white);
        water_cube.calculate_tangents();
        let water_vertices = convert_mesh_to_gpu(&water_cube);
        mesh_manager.upload_mesh(&renderer.device, "water_cube".to_string(), &water_vertices, &water_cube.indices);

        // Scene is now loaded from file instead of being generated
        // No need to create entities programmatically

        // Initialize physics world
        let mut physics_world = PhysicsWorld::default(); // Default gravity is (0, -9.81, 0)
        PhysicsSync::initialize_physics(&mut physics_world, &scene)?;

        // Initialize audio system
        let assets_path = std::env::current_dir()?.join("assets");
        let audio_system = AudioSystem::new(assets_path)?;
        log::info!("Audio system initialized");

        // Initialize script system
        let mut script_system = ScriptSystem::new();
        script_system.initialize(&scene)?;
        script_system.start(&mut scene)?;
        // Register audio API with scripts
        script_system.register_audio_api(self.audio_command_queue.clone());
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

        // Create particle renderer
        let particle_renderer = ParticleRenderer::new(
            &renderer.device,
            renderer.surface_config.format,
            engine_render::particle_renderer::ParticleBlendMode::Alpha,
        ).ok();

        self.window = Some(window.clone());
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            mesh_manager,
            texture_manager,
            material_manager,
            depth_texture,
            skybox,
            shadow_map,
            camera_bind_group_layout,
            camera_bind_group: Some(camera_bind_group),
            camera_uniform_buffer,
            framebuffer,
            post_process_pipeline,
            particle_renderer,
            particle_systems: std::collections::HashMap::new(),
            particle_compute_pipelines: std::collections::HashMap::new(),
        });
        self.camera = Some(camera);
        self.scene = Some(scene);
        self.asset_manager = Some(asset_manager);
        self.physics_world = Some(physics_world);
        self.script_system = Some(script_system);
        self.audio_system = Some(audio_system);
        self.entity_ids = Vec::new(); // Scene loaded from file, not tracking individual entity IDs
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
        let Some(wgpu_state) = &mut self.wgpu_state else {
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
        let Some(asset_manager) = &mut self.asset_manager else {
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
                    ReloadEvent::ModelChanged(path) => {
                        log::info!("Model changed: {:?}", path);
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
                    ReloadEvent::AssetChanged(path) => {
                        log::info!("Generic asset changed: {:?}", path);
                        // Generic asset change - determine type by extension
                        let path_str = path.to_string_lossy();
                        if path_str.ends_with(".png") || path_str.ends_with(".jpg") ||
                           path_str.ends_with(".jpeg") || path_str.ends_with(".bmp") {
                            // Treat as texture
                            if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                let rel_str = relative_path.to_string_lossy();
                                let _ = asset_manager.reload_texture(&rel_str);
                            }
                        } else if path_str.ends_with(".gltf") || path_str.ends_with(".glb") {
                            // Treat as model
                            if let Ok(relative_path) = path.strip_prefix(asset_manager.asset_root()) {
                                let rel_str = relative_path.to_string_lossy();
                                let _ = asset_manager.reload_gltf(&rel_str);
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

        // Update audio system
        if let Some(audio_system) = &mut self.audio_system {
            // Process audio commands from scripts
            let mut commands = self.audio_command_queue.lock().unwrap();
            for command in commands.drain(..) {
                match command {
                    AudioCommand::PlaySound { path, volume } => {
                        if let Err(e) = audio_system.play_sound(&path, volume) {
                            log::warn!("Failed to play sound '{}': {}", path, e);
                        }
                    }
                    AudioCommand::PlayMusic { path, volume, looping } => {
                        if let Err(e) = audio_system.play_music(&path, volume, looping) {
                            log::warn!("Failed to play music '{}': {}", path, e);
                        }
                    }
                    AudioCommand::StopMusic => {
                        audio_system.stop_music();
                    }
                }
            }
            drop(commands); // Release the lock

            // Create audio listener from camera transform
            let listener = engine_audio::AudioListener::from_transform(camera.position, Quat::IDENTITY);

            // Process AudioSource components
            for entity in scene.entities() {
                if let Some(audio_source) = entity.get_component::<AudioSource>() {
                    let position = entity.transform.position;

                    // Play audio if marked to play on start and not currently playing
                    if audio_source.play_on_start && !audio_source.playing {
                        // Play 3D sound
                        if let Err(e) = audio_system.play_3d_sound(
                            &audio_source.audio_path,
                            position,
                            &listener,
                            audio_source.volume,
                            audio_source.max_distance,
                        ) {
                            log::warn!("Failed to play audio '{}': {}", audio_source.audio_path, e);
                        } else {
                            // Mark as playing (note: this is read-only view, so we can't actually update it)
                            // TODO: Need mutable access to components to track playing state
                        }
                    }
                }
            }
        }

        // Initialize and update particle systems
        for entity in scene.entities() {
            if let Some(particle_emitter) = entity.get_component::<ParticleEmitter>() {
                if !particle_emitter.enabled {
                    continue;
                }

                let entity_id = entity.id;

                // Initialize particle system if it doesn't exist
                if !wgpu_state.particle_systems.contains_key(&entity_id) {
                    use engine_particles::{ParticleSystem, EmitterProperties, EmitterShape};

                    // Parse emitter shape
                    let shape = match particle_emitter.shape.as_str() {
                        "sphere" => EmitterShape::Sphere { radius: 1.0 },
                        "cone" => EmitterShape::Cone { angle: 30.0, radius: 1.0 },
                        "box" => EmitterShape::Box { size: Vec3::ONE },
                        "circle" => EmitterShape::Circle { radius: 1.0 },
                        _ => EmitterShape::Point,
                    };

                    let properties = EmitterProperties {
                        shape,
                        rate: particle_emitter.rate,
                        initial_velocity: Vec3::from(particle_emitter.initial_velocity),
                        velocity_randomness: particle_emitter.velocity_randomness,
                        lifetime: particle_emitter.lifetime,
                        lifetime_randomness: particle_emitter.lifetime_randomness,
                        initial_size: particle_emitter.initial_size,
                        size_over_lifetime: vec![1.0, 0.1], // Linear shrink
                        initial_color: particle_emitter.initial_color,
                        color_over_lifetime: vec![],
                        gravity: Vec3::from(particle_emitter.gravity),
                    };

                    let position = entity.transform.position;
                    let mut system = ParticleSystem::new(particle_emitter.max_particles, properties);
                    system.position = position;

                    wgpu_state.particle_systems.insert(entity_id, system);

                    // Create compute pipeline for this particle system
                    if let Ok(compute_pipeline) = engine_particles::ParticleComputePipeline::new(
                        &wgpu_state.renderer.device,
                        particle_emitter.max_particles,
                        &[], // Initial particles (empty)
                    ) {
                        wgpu_state.particle_compute_pipelines.insert(entity_id, compute_pipeline);
                    }
                }

                // Update particle system position from entity transform
                if let Some(system) = wgpu_state.particle_systems.get_mut(&entity_id) {
                    system.position = entity.transform.position;
                    system.update(dt);
                }
            }
        }

        // Begin frame
        let (output, mut encoder, view) = wgpu_state.renderer.begin_frame(
            &wgpu_state.surface,
            &wgpu_state.depth_texture,
        )?;

        // Update and dispatch particle compute shaders
        for (entity_id, particle_system) in &wgpu_state.particle_systems {
            if let Some(compute_pipeline) = wgpu_state.particle_compute_pipelines.get(entity_id) {
                // Upload current particle data to GPU
                compute_pipeline.upload_particles(
                    &wgpu_state.renderer.queue,
                    &particle_system.particles,
                );

                // Update simulation uniforms
                compute_pipeline.update_uniforms(
                    &wgpu_state.renderer.queue,
                    dt,
                    self.time,
                    particle_system.properties.gravity,
                );

                // Dispatch compute shader
                compute_pipeline.dispatch(&mut encoder);
            }
        }

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

            // Update shadow uniforms once (light space matrix only)
            shadow_map.update_uniforms(
                &wgpu_state.renderer.queue,
                light_space_matrix,
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
                shadow_pass.set_bind_group(0, &shadow_map.bind_group, &[]);

                // Render all meshes to shadow map
                for entity in scene.entities() {
                    if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                        if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&mesh_renderer.mesh_path) {
                            if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                                let world_matrix = scene.world_matrix(entity.id);

                                // Set push constants for model matrix
                                use engine_render::shadow::ShadowPushConstants;
                                let push_constants = ShadowPushConstants {
                                    model: world_matrix.to_cols_array_2d(),
                                };
                                shadow_pass.set_push_constants(
                                    wgpu::ShaderStages::VERTEX,
                                    0,
                                    bytemuck::cast_slice(&[push_constants]),
                                );

                                shadow_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                                shadow_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                shadow_pass.draw_indexed(0..gpu_mesh.num_indices, 0, 0..1);
                            }
                        }
                    }
                }
            } // shadow_pass dropped here
        }

        // Create shadow sampling bind group for main render pass
        let shadow_sampling_bind_group = if let Some(ref shadow_map) = wgpu_state.shadow_map {
            let layout = ShadowMap::create_sampling_bind_group_layout(&wgpu_state.renderer.device);
            Some(shadow_map.create_sampling_bind_group(&wgpu_state.renderer.device, &layout))
        } else {
            None
        };

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

        // Render all entities with textures
        let mut first_mesh = wgpu_state.skybox.is_none();
        for entity in scene.entities() {
            if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&mesh_renderer.mesh_path) {
                    if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                        let world_matrix = scene.world_matrix(entity.id);

                        // Get material path (use default if not specified)
                        let material_path = mesh_renderer.material_path.as_deref()
                            .unwrap_or("materials/default.mat");

                        // Get or upload material
                        let material_handle = if let Some(handle) = wgpu_state.material_manager.get_handle(material_path) {
                            handle
                        } else {
                            // Load material from asset manager
                            let material_handle = asset_manager.load_material(material_path)
                                .unwrap_or_else(|e| {
                                    log::warn!("Failed to load material '{}': {}, using default", material_path, e);
                                    AssetHandle::new(Material::default())
                                });
                            let material = material_handle.inner.as_ref();

                            // Load required textures
                            let albedo_handle = material.albedo_texture.as_ref()
                                .and_then(|path| {
                                    asset_manager.load_texture(path).ok()
                                        .map(|tex_handle| wgpu_state.texture_manager.upload_texture(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, path.to_string(), tex_handle.inner.as_ref()))
                                })
                                .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());

                            let normal_handle = material.normal_texture.as_ref()
                                .and_then(|path| {
                                    asset_manager.load_texture(path).ok()
                                        .map(|tex_handle| wgpu_state.texture_manager.upload_texture(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, path.to_string(), tex_handle.inner.as_ref()))
                                })
                                .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());

                            let metallic_roughness_handle = material.metallic_roughness_texture.as_ref()
                                .and_then(|path| {
                                    asset_manager.load_texture(path).ok()
                                        .map(|tex_handle| wgpu_state.texture_manager.upload_texture(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, path.to_string(), tex_handle.inner.as_ref()))
                                })
                                .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());

                            let ao_handle = material.ao_texture.as_ref()
                                .and_then(|path| {
                                    asset_manager.load_texture(path).ok()
                                        .map(|tex_handle| wgpu_state.texture_manager.upload_texture(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, path.to_string(), tex_handle.inner.as_ref()))
                                })
                                .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());

                            // Upload material to GPU
                            wgpu_state.material_manager.upload_material(
                                &wgpu_state.renderer.device,
                                &wgpu_state.texture_manager,
                                material_path.to_string(),
                                &material,
                                albedo_handle,
                                Some(normal_handle),
                                Some(metallic_roughness_handle),
                                Some(ao_handle),
                            )
                        };

                        // Get material bind group
                        let material_bind_group = wgpu_state.material_manager.get_material(material_handle)
                            .map(|mat| &mat.bind_group)
                            .expect("Material bind group should exist");

                        // Get shadow bind group (or create a dummy one if shadows disabled)
                        let shadow_bind_group = if let Some(ref shadow_bg) = shadow_sampling_bind_group {
                            shadow_bg
                        } else {
                            // Create a fallback bind group if shadows are disabled
                            // (this shouldn't happen since shadow_map is created in init)
                            continue;
                        };

                        wgpu_state.renderer.render_mesh(
                            &mut encoder,
                            &view,
                            &wgpu_state.depth_texture,
                            gpu_mesh,
                            view_proj,
                            camera.position,
                            world_matrix,
                            material_bind_group,
                            shadow_bind_group,
                            first_mesh,
                        );
                        first_mesh = false;
                    }
                }
            }
        }

        // Render particles after opaque geometry
        if let Some(ref particle_renderer) = wgpu_state.particle_renderer {
            // Calculate camera basis vectors for billboard rendering
            let camera_forward = (camera.target - camera.position).normalize();
            let camera_right = camera_forward.cross(camera.up).normalize();
            let camera_up = camera_right.cross(camera_forward).normalize();

            // Update particle renderer camera uniforms
            particle_renderer.update_camera(
                &wgpu_state.renderer.queue,
                view_proj,
                camera.position,
                camera_right,
                camera_up,
            );

            // Render each particle system
            for (entity_id, particle_system) in &wgpu_state.particle_systems {
                if let Some(compute_pipeline) = wgpu_state.particle_compute_pipelines.get(entity_id) {
                    let particle_count = particle_system.particles.len() as u32;
                    if particle_count > 0 {
                        let mut particle_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Particle Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load, // Don't clear - preserve geometry
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &wgpu_state.depth_texture,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load, // Preserve depth
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        particle_renderer.render(
                            &mut particle_pass,
                            compute_pipeline.particle_buffer(),
                            particle_count,
                        );
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check if exit was requested from UI
        if let Some(ui) = &self.ui {
            if ui.is_exit_requested() {
                log::info!("Exit requested by user");
                event_loop.exit();
                return;
            }
        }

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

    // Parse command line arguments
    let args = Args::parse();

    // Determine scene file path (use provided or default to castle scene)
    let scene_file = args.scene.or_else(|| Some("assets/scenes/castle.ron".to_string()));

    if let Some(ref path) = scene_file {
        log::info!("Will load scene from: {}", path);
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new(scene_file);
    event_loop.run_app(&mut app)?;

    Ok(())
}
