// Causality Engine - Editor

mod ui;
pub mod ipc;
mod file_ipc;
mod undo;

use anyhow::Result;
use undo::UndoHistory;
use clap::Parser;
use engine_assets::{manager::{AssetHandle, AssetManager}, material::Material, mesh::Mesh, texture::Texture, HotReloadWatcher, ReloadEvent, HeightMap, TerrainConfig, Terrain, compute_water_fill, generate_water_mesh, vegetation::VegetationType};
use wgpu::util::DeviceExt;
use engine_audio::AudioSystem;
use engine_physics::{Collider, PhysicsSync, PhysicsWorld, RigidBody, BuoyancySystem, WaterVolume};
use engine_render::{
    camera::Camera,
    foliage_renderer::{FoliageRenderer, FoliageInstanceGpu, FoliageRenderData},
    frustum::AABB,
    gpu_mesh::GpuVertex,
    material_manager::MaterialManager,
    mesh_manager::MeshManager,
    particle_renderer::ParticleRenderer,
    postprocess::{Framebuffer, PostProcessPipeline},
    renderer::Renderer,
    shadow::ShadowMap,
    skybox::Skybox,
    texture_manager::TextureManager,
    water::WaterRenderer,
};
use engine_scene::{
    components::{AudioListener, AudioSource, MeshRenderer, ParticleEmitter, Water, TerrainWater, WaterBody, TerrainGenerator, Foliage, FoliageInstance},
    entity::EntityId,
    scene::Scene,
    transform::Transform,
};
use engine_scripting::{AudioCommand, AudioCommandQueue, Script, ScriptSystem};
use glam::{Quat, Vec3, Vec4};
use std::sync::{Arc, Mutex};
use ui::{viewport::ViewportControls, EditorUi, EditorResult, BrushMode};
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
    buoyancy_system: Option<BuoyancySystem>,
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
    modifiers: winit::keyboard::ModifiersState,
    undo_history: UndoHistory,
    /// Clipboard for copy/paste of entities
    clipboard: Option<engine_scene::scene_data::SerializedEntity>,
    /// Terrain heightmap undo stack
    terrain_undo_stack: Vec<Vec<f32>>,
    terrain_redo_stack: Vec<Vec<f32>>,
    /// Flag to track if we're currently sculpting (to save undo state on first brush stroke)
    terrain_sculpt_started: bool,
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
    msaa_texture: wgpu::TextureView,
    skybox: Option<Skybox>,
    shadow_map: Option<ShadowMap>,
    water_renderer: Option<WaterRenderer>,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: Option<wgpu::BindGroup>,
    camera_uniform_buffer: wgpu::Buffer,
    framebuffer: Option<Framebuffer>,
    post_process_pipeline: Option<PostProcessPipeline>,
    particle_renderer: Option<ParticleRenderer>,
    particle_systems: std::collections::HashMap<EntityId, engine_particles::ParticleSystem>,
    particle_compute_pipelines: std::collections::HashMap<EntityId, engine_particles::ParticleComputePipeline>,
    /// Foliage renderer for instanced vegetation
    foliage_renderer: Option<FoliageRenderer>,
    /// Terrain heightmap for terrain-aware water
    terrain_heightmap: Option<HeightMap>,
    terrain_config: Option<TerrainConfig>,
    /// Computed terrain water bodies for rendering
    terrain_water_bodies: Vec<TerrainWaterBodyInfo>,
    /// Flag to regenerate terrain on next frame
    terrain_needs_regeneration: bool,
    /// Flag to regenerate water on next frame
    water_needs_regeneration: bool,
}

/// Info about a computed terrain water body for rendering
struct TerrainWaterBodyInfo {
    mesh_name: String,
    surface_level: f32,
    flow_direction: Option<[f32; 2]>,
    flow_speed: f32,
}

// Uniforms and push constants now handled by renderer
// No need to redefine here since render_mesh handles it

/// Raycast against terrain heightmap to find hit point
/// Returns Some(hit_position) if the ray intersects the terrain
fn raycast_terrain(
    ray_origin: Vec3,
    ray_direction: Vec3,
    heightmap: &HeightMap,
    config: &TerrainConfig,
) -> Option<Vec3> {
    // Step along the ray
    let step_size = 0.5;
    let max_distance = 200.0;

    let mut t = 0.0;
    let mut prev_point = ray_origin;
    let mut prev_height = heightmap.sample_height(prev_point.x, prev_point.z, config.scale);
    let mut prev_above = prev_point.y > prev_height;

    while t < max_distance {
        t += step_size;
        let point = ray_origin + ray_direction * t;

        // Sample terrain height at this XZ position
        let terrain_height = heightmap.sample_height(point.x, point.z, config.scale);

        let above = point.y > terrain_height;

        // Check if we crossed the terrain surface
        if prev_above && !above {
            // Linear interpolation to find more precise hit point
            let blend = (prev_point.y - prev_height) / ((prev_point.y - prev_height) + (terrain_height - point.y));
            let hit_point = prev_point + (point - prev_point) * blend;

            // Clamp to terrain bounds
            let half_width = (config.width as f32 * config.scale) / 2.0;
            let half_depth = (config.depth as f32 * config.scale) / 2.0;
            if hit_point.x >= -half_width && hit_point.x <= half_width
                && hit_point.z >= -half_depth && hit_point.z <= half_depth
            {
                return Some(hit_point);
            }
        }

        prev_point = point;
        prev_height = terrain_height;
        prev_above = above;
    }

    None
}

/// Pick an entity by raycasting against entity bounding boxes
/// Returns the EntityId and distance of the closest entity hit, if any
fn pick_entity(
    ray_origin: Vec3,
    ray_direction: Vec3,
    scene: &Scene,
) -> Option<(EntityId, f32)> {
    let mut closest: Option<(EntityId, f32)> = None;

    for entity in scene.entities() {
        // Skip entities without mesh renderers (they have no visual representation)
        if !entity.has_component::<MeshRenderer>() {
            continue;
        }

        // Create bounding box from entity transform
        // Use scale as half-extents for a rough bounding box
        let pos = entity.transform.position;
        let scale = entity.transform.scale;

        // Create AABB centered at entity position with scale as half-extents
        let aabb = AABB::from_center_extents(pos, scale);

        // Test ray intersection
        if let Some(t) = aabb.ray_intersect(ray_origin, ray_direction) {
            // Check if this is closer than the current closest hit
            if let Some((_, closest_t)) = closest {
                if t < closest_t {
                    closest = Some((entity.id, t));
                }
            } else {
                closest = Some((entity.id, t));
            }
        }
    }

    closest
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
            buoyancy_system: None,
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
            modifiers: winit::keyboard::ModifiersState::empty(),
            file_ipc: Some(file_ipc::FileIpcHandler::new()),
            scene_file_path,
            undo_history: UndoHistory::new(50),
            clipboard: None,
            terrain_undo_stack: Vec::new(),
            terrain_redo_stack: Vec::new(),
            terrain_sculpt_started: false,
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
        let msaa_texture = renderer.create_msaa_texture(size.width, size.height, renderer.surface_config.format);
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

        // Process TerrainGenerator and TerrainWater components
        let mut terrain_heightmap = None;
        let mut terrain_config = None;
        let mut terrain_water_bodies = Vec::new();

        // First, look for TerrainGenerator component to get terrain config
        for entity in scene.entities() {
            if let Some(terrain_gen) = entity.get_component::<TerrainGenerator>() {
                log::info!("Found TerrainGenerator component: {}x{}, scale={}, moat={}",
                    terrain_gen.width, terrain_gen.depth, terrain_gen.scale, terrain_gen.moat_enabled);

                // Build TerrainConfig from component
                let config = TerrainConfig {
                    width: terrain_gen.width,
                    depth: terrain_gen.depth,
                    scale: terrain_gen.scale,
                    height_scale: terrain_gen.height_scale,
                    seed: terrain_gen.seed,
                    octaves: terrain_gen.octaves,
                    frequency: terrain_gen.frequency,
                    lacunarity: terrain_gen.lacunarity,
                    persistence: terrain_gen.persistence,
                };

                // Generate heightmap based on moat setting
                let heightmap = if terrain_gen.moat_enabled {
                    HeightMap::generate_with_moat(
                        &config,
                        terrain_gen.moat_inner_radius,
                        terrain_gen.moat_outer_radius,
                        terrain_gen.moat_depth,
                    )
                } else {
                    HeightMap::generate(&config)
                };

                log::info!("Generated terrain heightmap {}x{}", config.width, config.depth);

                // Generate and upload terrain mesh
                let terrain_mesh = Terrain::generate_mesh_from_heightmap(&heightmap, &config);
                log::info!("Generated terrain mesh '{}' with {} vertices", terrain_mesh.name, terrain_mesh.vertices.len());
                let gpu_vertices = convert_mesh_to_gpu(&terrain_mesh);
                mesh_manager.upload_mesh(&renderer.device, "terrain".to_string(), &gpu_vertices, &terrain_mesh.indices);

                terrain_heightmap = Some(heightmap);
                terrain_config = Some(config);
                break; // Only process first TerrainGenerator
            }
        }

        // Then, process TerrainWater components for water fill
        for entity in scene.entities() {
            if let Some(terrain_water) = entity.get_component::<TerrainWater>() {
                log::info!("Found TerrainWater component with ground_water_level={}", terrain_water.ground_water_level);

                // Compute water fill if we have terrain
                if let (Some(ref heightmap), Some(ref config)) = (&terrain_heightmap, &terrain_config) {
                    let result = compute_water_fill(
                        heightmap,
                        terrain_water.ground_water_level,
                        terrain_water.min_water_depth,
                        terrain_water.min_water_area,
                    );
                    log::info!("Computed water fill: {} water bodies found", result.water_bodies.len());

                    // Generate and upload meshes for each water body
                    for computed_body in &result.water_bodies {
                        let mesh = generate_water_mesh(computed_body, heightmap, config);
                        log::info!("Generated water mesh '{}' with {} vertices, {} indices",
                            mesh.name, mesh.vertices.len(), mesh.indices.len());

                        let gpu_vertices = convert_mesh_to_gpu(&mesh);
                        mesh_manager.upload_mesh(&renderer.device, mesh.name.clone(), &gpu_vertices, &mesh.indices);

                        // Store water body info for rendering
                        terrain_water_bodies.push(TerrainWaterBodyInfo {
                            mesh_name: mesh.name,
                            surface_level: computed_body.surface_level,
                            flow_direction: computed_body.flow_direction,
                            flow_speed: computed_body.flow_speed,
                        });
                    }
                }
            }
        }

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
            &renderer.queue,
            renderer.surface_config.format,
            engine_render::particle_renderer::ParticleBlendMode::Alpha,
        ).ok();

        // Create foliage renderer for instanced vegetation
        let foliage_renderer = FoliageRenderer::new(
            &renderer.device,
            renderer.surface_config.format,
        ).ok();

        // Generate and upload vegetation meshes
        for veg_type in VegetationType::all() {
            let mesh = veg_type.generate_mesh();
            let gpu_vertices = convert_mesh_to_gpu(&mesh);
            mesh_manager.upload_mesh(&renderer.device, veg_type.mesh_name().to_string(), &gpu_vertices, &mesh.indices);
            log::info!("Generated vegetation mesh '{}' with {} vertices", veg_type.mesh_name(), mesh.vertices.len());
        }

        // Create water renderer
        let water_renderer = if let Some(ref shadow_map) = shadow_map {
            let shadow_sampling_layout = ShadowMap::create_sampling_bind_group_layout(&renderer.device);
            WaterRenderer::new(
                &renderer.device,
                renderer.surface_config.format,
                texture_manager.bind_group_layout(),
                &shadow_sampling_layout,
            ).ok()
        } else {
            None
        };

        self.window = Some(window.clone());
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            mesh_manager,
            texture_manager,
            material_manager,
            depth_texture,
            msaa_texture,
            skybox,
            shadow_map,
            water_renderer,
            camera_bind_group_layout,
            camera_bind_group: Some(camera_bind_group),
            camera_uniform_buffer,
            framebuffer,
            post_process_pipeline,
            particle_renderer,
            particle_systems: std::collections::HashMap::new(),
            particle_compute_pipelines: std::collections::HashMap::new(),
            foliage_renderer,
            terrain_heightmap,
            terrain_config,
            terrain_water_bodies,
            terrain_needs_regeneration: false,
            water_needs_regeneration: false,
        });
        self.camera = Some(camera);
        self.scene = Some(scene);
        self.asset_manager = Some(asset_manager);
        self.physics_world = Some(physics_world);
        self.buoyancy_system = Some(BuoyancySystem::new());
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
                wgpu_state.msaa_texture = wgpu_state.renderer.create_msaa_texture(new_size.width, new_size.height, wgpu_state.renderer.surface_config.format);
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

        // Update elapsed time for animations
        self.time += dt;

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

        // Handle deferred terrain/water regeneration
        if wgpu_state.terrain_needs_regeneration {
            wgpu_state.terrain_needs_regeneration = false;

            // Find TerrainGenerator component and regenerate
            for entity in scene.entities() {
                if let Some(terrain_gen) = entity.get_component::<TerrainGenerator>() {
                    log::info!("Regenerating terrain: {}x{}, moat={}", terrain_gen.width, terrain_gen.depth, terrain_gen.moat_enabled);

                    let config = TerrainConfig {
                        width: terrain_gen.width,
                        depth: terrain_gen.depth,
                        scale: terrain_gen.scale,
                        height_scale: terrain_gen.height_scale,
                        seed: terrain_gen.seed,
                        octaves: terrain_gen.octaves,
                        frequency: terrain_gen.frequency,
                        lacunarity: terrain_gen.lacunarity,
                        persistence: terrain_gen.persistence,
                    };

                    let heightmap = if terrain_gen.moat_enabled {
                        HeightMap::generate_with_moat(
                            &config,
                            terrain_gen.moat_inner_radius,
                            terrain_gen.moat_outer_radius,
                            terrain_gen.moat_depth,
                        )
                    } else {
                        HeightMap::generate(&config)
                    };

                    let terrain_mesh = Terrain::generate_mesh_from_heightmap(&heightmap, &config);
                    let gpu_vertices = convert_mesh_to_gpu(&terrain_mesh);
                    wgpu_state.mesh_manager.upload_mesh(&wgpu_state.renderer.device, "terrain".to_string(), &gpu_vertices, &terrain_mesh.indices);

                    wgpu_state.terrain_heightmap = Some(heightmap);
                    wgpu_state.terrain_config = Some(config);

                    if let Some(ui) = &mut self.ui {
                        ui.log_info("Terrain regenerated".to_string());
                    }
                    break;
                }
            }
        }

        if wgpu_state.water_needs_regeneration {
            wgpu_state.water_needs_regeneration = false;

            // Find TerrainWater component and regenerate water
            if let (Some(ref heightmap), Some(ref config)) = (&wgpu_state.terrain_heightmap, &wgpu_state.terrain_config) {
                wgpu_state.terrain_water_bodies.clear();

                for entity in scene.entities() {
                    if let Some(terrain_water) = entity.get_component::<TerrainWater>() {
                        log::info!("Regenerating water: ground_level={}", terrain_water.ground_water_level);

                        let result = compute_water_fill(
                            heightmap,
                            terrain_water.ground_water_level,
                            terrain_water.min_water_depth,
                            terrain_water.min_water_area,
                        );
                        log::info!("Water fill: {} bodies found", result.water_bodies.len());

                        for computed_body in &result.water_bodies {
                            let mesh = generate_water_mesh(computed_body, heightmap, config);
                            let gpu_vertices = convert_mesh_to_gpu(&mesh);
                            wgpu_state.mesh_manager.upload_mesh(&wgpu_state.renderer.device, mesh.name.clone(), &gpu_vertices, &mesh.indices);

                            wgpu_state.terrain_water_bodies.push(TerrainWaterBodyInfo {
                                mesh_name: mesh.name,
                                surface_level: computed_body.surface_level,
                                flow_direction: computed_body.flow_direction,
                                flow_speed: computed_body.flow_speed,
                            });
                        }

                        if let Some(ui) = &mut self.ui {
                            ui.log_info(format!("Water regenerated: {} bodies", result.water_bodies.len()));
                        }
                        break;
                    }
                }
            }
        }

        // Update camera from viewport controls
        self.viewport_controls.update_camera(camera);

        // Update camera info in UI for status bar
        if let Some(ui) = &mut self.ui {
            ui.update_camera_info(camera.position, self.viewport_controls.orbit_distance);
        }

        // Handle terrain sculpting (separate borrow scope for mutable access)
        if let Some(ui) = &self.ui {
            if ui.brush_tool.mode.is_terrain_mode() && self.viewport_controls.brush_held {
                let brush_tool = ui.brush_tool.clone();

                // Save undo state when starting to sculpt (first brush stroke)
                if !self.terrain_sculpt_started {
                    if let Some(ref heightmap) = wgpu_state.terrain_heightmap {
                        // Save current heightmap state for undo
                        self.terrain_undo_stack.push(heightmap.heights.clone());
                        // Limit undo stack size
                        if self.terrain_undo_stack.len() > 20 {
                            self.terrain_undo_stack.remove(0);
                        }
                        // Clear redo stack on new edit
                        self.terrain_redo_stack.clear();
                    }
                    self.terrain_sculpt_started = true;
                }

                // Get screen dimensions
                let screen_width = wgpu_state.renderer.surface_config.width as f32;
                let screen_height = wgpu_state.renderer.surface_config.height as f32;
                let (mouse_x, mouse_y) = self.viewport_controls.current_mouse_pos;

                // Convert screen position to ray
                let (ray_origin, ray_direction) = camera.screen_to_ray(mouse_x, mouse_y, screen_width, screen_height);

                if let (Some(ref mut heightmap), Some(ref config)) = (&mut wgpu_state.terrain_heightmap, &wgpu_state.terrain_config) {
                    if let Some(hit_point) = raycast_terrain(ray_origin, ray_direction, heightmap, config) {
                        if let Some(terrain_mode) = brush_tool.mode.terrain_mode_code() {
                            // Check if we should apply (throttle to avoid too many updates)
                            let should_sculpt = if let Some((last_x, last_z)) = self.viewport_controls.last_terrain_sculpt_pos {
                                let dx = hit_point.x - last_x;
                                let dz = hit_point.z - last_z;
                                let dist = (dx * dx + dz * dz).sqrt();
                                // Only apply if moved more than 1/4 of brush radius
                                dist > brush_tool.radius * 0.25
                            } else {
                                true
                            };

                            if should_sculpt {
                                let modified = heightmap.apply_brush(
                                    hit_point.x,
                                    hit_point.z,
                                    config.scale,
                                    brush_tool.radius,
                                    brush_tool.terrain_strength,
                                    terrain_mode,
                                );

                                if modified {
                                    // Regenerate terrain mesh
                                    let terrain_mesh = Terrain::generate_mesh_from_heightmap(heightmap, config);
                                    let gpu_vertices = convert_mesh_to_gpu(&terrain_mesh);
                                    wgpu_state.mesh_manager.upload_mesh(
                                        &wgpu_state.renderer.device,
                                        "terrain".to_string(),
                                        &gpu_vertices,
                                        &terrain_mesh.indices,
                                    );

                                    // Update last sculpt position
                                    self.viewport_controls.last_terrain_sculpt_pos = Some((hit_point.x, hit_point.z));

                                    // Mark scene as modified
                                    if let Some(ui) = self.ui.as_mut() {
                                        ui.mark_scene_modified();
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Reset sculpt started flag when not sculpting
                self.terrain_sculpt_started = false;
            }
        }

        // Handle vegetation brush tool placement (separate borrow scope for immutable access)
        if let Some(ui) = &self.ui {
            if ui.brush_tool.mode.is_vegetation_mode() && self.viewport_controls.brush_active {
                // Get screen dimensions
                let screen_width = wgpu_state.renderer.surface_config.width as f32;
                let screen_height = wgpu_state.renderer.surface_config.height as f32;
                let (mouse_x, mouse_y) = self.viewport_controls.current_mouse_pos;

                // Convert screen position to ray
                let (ray_origin, ray_direction) = camera.screen_to_ray(mouse_x, mouse_y, screen_width, screen_height);

                // Raycast to terrain
                if let (Some(ref heightmap), Some(ref config)) = (&wgpu_state.terrain_heightmap, &wgpu_state.terrain_config) {
                    if let Some(hit_point) = raycast_terrain(ray_origin, ray_direction, heightmap, config) {
                        let brush_tool = ui.brush_tool.clone();
                        // ui borrow ends here, brush_tool is a clone

                        match brush_tool.mode {
                            BrushMode::Place => {
                                // Get vegetation mesh name
                                let veg_mesh_name = match brush_tool.vegetation_type {
                                    crate::ui::VegetationType::PineTree => "vegetation_pine",
                                    crate::ui::VegetationType::OakTree => "vegetation_oak",
                                    crate::ui::VegetationType::Bush => "vegetation_bush",
                                    crate::ui::VegetationType::Shrub => "vegetation_shrub",
                                };

                                // Find or create foliage entity for this vegetation type
                                let mut foliage_entity_id = None;
                                for entity in scene.entities() {
                                    if let Some(foliage) = entity.get_component::<Foliage>() {
                                        if foliage.vegetation_type == veg_mesh_name {
                                            foliage_entity_id = Some(entity.id);
                                            break;
                                        }
                                    }
                                }

                                let entity_id = if let Some(id) = foliage_entity_id {
                                    id
                                } else {
                                    // Create new foliage entity
                                    let new_id = scene.create_entity(format!("Foliage - {}", brush_tool.vegetation_type.name()));
                                    if let Some(entity) = scene.get_entity_mut(new_id) {
                                        entity.add_component(Foliage {
                                            vegetation_type: veg_mesh_name.to_string(),
                                            instances: Vec::new(),
                                            cast_shadows: true,
                                            color_tint: [1.0, 1.0, 1.0],
                                        });
                                    }
                                    new_id
                                };

                                // Add instances based on brush settings
                                if let Some(entity) = scene.get_entity_mut(entity_id) {
                                    if let Some(foliage) = entity.get_component_mut::<Foliage>() {
                                        // Place instances within brush radius
                                        let instance_count = brush_tool.density.ceil() as usize;
                                        let mut rng_seed = (hit_point.x * 1000.0 + hit_point.z * 100.0) as u64;

                                        for _ in 0..instance_count {
                                            // Simple pseudo-random within radius
                                            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
                                            let r1 = ((rng_seed >> 16) & 0xFFFF) as f32 / 65535.0;
                                            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
                                            let r2 = ((rng_seed >> 16) & 0xFFFF) as f32 / 65535.0;
                                            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
                                            let r3 = ((rng_seed >> 16) & 0xFFFF) as f32 / 65535.0;
                                            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
                                            let r4 = ((rng_seed >> 16) & 0xFFFF) as f32 / 65535.0;

                                            // Random offset within radius (uniform disk distribution)
                                            let angle = r1 * std::f32::consts::TAU;
                                            let dist = (r2.sqrt()) * brush_tool.radius;
                                            let offset_x = angle.cos() * dist;
                                            let offset_z = angle.sin() * dist;

                                            // Sample terrain height at the offset position
                                            let instance_x = hit_point.x + offset_x;
                                            let instance_z = hit_point.z + offset_z;
                                            let instance_y = heightmap.sample_height(instance_x, instance_z, config.scale);

                                            // Random scale and rotation
                                            let scale = brush_tool.scale_min + r3 * (brush_tool.scale_max - brush_tool.scale_min);
                                            let rotation_y = if brush_tool.random_rotation {
                                                r4 * std::f32::consts::TAU
                                            } else {
                                                0.0
                                            };

                                            foliage.instances.push(FoliageInstance {
                                                position: [instance_x, instance_y, instance_z],
                                                rotation_y,
                                                scale,
                                            });
                                        }
                                    }
                                }

                                if let Some(ui) = self.ui.as_mut() {
                                    ui.mark_scene_modified();
                                }
                            }
                            BrushMode::Erase => {
                                // Remove instances near hit point
                                let erase_radius_sq = brush_tool.radius * brush_tool.radius;

                                // Collect changes first (to avoid borrow conflict)
                                let mut updates: Vec<(EntityId, Vec<FoliageInstance>)> = Vec::new();

                                for entity in scene.entities() {
                                    if let Some(foliage) = entity.get_component::<Foliage>() {
                                        let entity_id = entity.id;
                                        let mut instances = foliage.instances.clone();
                                        let original_count = instances.len();

                                        instances.retain(|instance| {
                                            let dx = instance.position[0] - hit_point.x;
                                            let dz = instance.position[2] - hit_point.z;
                                            let dist_sq = dx * dx + dz * dz;
                                            dist_sq > erase_radius_sq
                                        });

                                        if instances.len() != original_count {
                                            updates.push((entity_id, instances));
                                        }
                                    }
                                }

                                // Apply updates
                                for (entity_id, instances) in updates {
                                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                                        if let Some(foliage) = entity.get_component_mut::<Foliage>() {
                                            foliage.instances = instances;
                                        }
                                    }
                                }

                                if let Some(ui) = self.ui.as_mut() {
                                    ui.mark_scene_modified();
                                }
                            }
                            // Select and terrain modes are not handled here
                            _ => {}
                        }

                        // Reset brush_active to prevent continuous vegetation placement
                        self.viewport_controls.brush_active = false;
                    }
                }
            }
        }

        // Handle entity selection by clicking in viewport (when in Select mode or brush panel hidden)
        if let Some(ui) = &self.ui {
            let in_select_mode = ui.brush_tool.mode == BrushMode::Select || !ui.show_brush_panel;
            if in_select_mode && self.viewport_controls.brush_active {
                // Get screen dimensions
                let screen_width = wgpu_state.renderer.surface_config.width as f32;
                let screen_height = wgpu_state.renderer.surface_config.height as f32;
                let (mouse_x, mouse_y) = self.viewport_controls.current_mouse_pos;

                // Convert screen position to ray
                let (ray_origin, ray_direction) = camera.screen_to_ray(mouse_x, mouse_y, screen_width, screen_height);

                // Try to pick an entity
                if let Some((entity_id, _distance)) = pick_entity(ray_origin, ray_direction, scene) {
                    if let Some(ui) = self.ui.as_mut() {
                        ui.selected_entity = Some(entity_id);
                        if let Some(entity) = scene.get_entity(entity_id) {
                            log::info!("Selected entity: {}", entity.name);
                        }
                    }
                } else {
                    // Clicked on nothing - deselect
                    if let Some(ui) = self.ui.as_mut() {
                        ui.selected_entity = None;
                    }
                }

                // Reset brush_active
                self.viewport_controls.brush_active = false;
            }
        }

        // Update scripts
        script_system.update(scene, dt)?;

        // Sync Water components to buoyancy system
        if let Some(buoyancy_system) = &mut self.buoyancy_system {
            // Clear and rebuild water volumes from Water components
            buoyancy_system.water_volumes.clear();
            for entity in scene.entities() {
                if let Some(water) = entity.get_component::<Water>() {
                    let transform = &entity.transform;
                    // Create water volume from Water component with flow
                    let flow_dir = Vec3::new(water.flow_direction[0], 0.0, water.flow_direction[1]);
                    let water_volume = WaterVolume::new(
                        transform.position,
                        transform.scale * 2.0, // Scale is half-extents, volume needs full size
                        transform.position.y + transform.scale.y, // Top of water
                    ).with_flow(flow_dir, water.flow_speed);
                    buoyancy_system.add_water_volume(water_volume);
                }
            }

            // Apply buoyancy forces to physics bodies
            buoyancy_system.update(&mut physics_world.rigid_body_set, scene);
        }

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

                    log::info!("Initializing particle emitter for entity {} ({})", entity_id.0, entity.name);
                    log::info!("  Rate: {}, Max particles: {}", particle_emitter.rate, particle_emitter.max_particles);

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

                    // Spawn initial batch of particles for immediate visibility
                    for _ in 0..50 {
                        system.update(0.016); // Simulate ~3 frames worth of spawning
                    }

                    log::info!("Created particle system at position {:?} with {} initial particles",
                              position, system.active_particle_count());

                    let initial_particles = system.particles.clone();
                    wgpu_state.particle_systems.insert(entity_id, system);

                    // Create compute pipeline with initial particles
                    // After this, GPU takes over - no more uploads!
                    if let Ok(compute_pipeline) = engine_particles::ParticleComputePipeline::new(
                        &wgpu_state.renderer.device,
                        particle_emitter.max_particles,
                        &initial_particles,
                    ) {
                        wgpu_state.particle_compute_pipelines.insert(entity_id, compute_pipeline);
                    }
                }

                // Update particle system and spawn new particles
                if let Some(system) = wgpu_state.particle_systems.get_mut(&entity_id) {
                    system.position = entity.transform.position;
                    system.update(dt);

                    // Upload particles to GPU (includes newly spawned ones)
                    // GPU compute will update them immediately after
                    if let Some(compute_pipeline) = wgpu_state.particle_compute_pipelines.get(&entity_id) {
                        compute_pipeline.upload_particles(
                            &wgpu_state.renderer.queue,
                            &system.particles,
                        );
                    }
                }
            }
        }

        // Begin frame
        let (output, mut encoder, view) = wgpu_state.renderer.begin_frame(
            &wgpu_state.surface,
            &wgpu_state.depth_texture,
        )?;

        // Update and dispatch particle compute shaders
        for (entity_id, particle_system) in &mut wgpu_state.particle_systems {
            if let Some(compute_pipeline) = wgpu_state.particle_compute_pipelines.get(entity_id) {
                // DON'T upload particles here - let GPU maintain state
                // Only upload uniforms for simulation

                // Update simulation uniforms
                compute_pipeline.update_uniforms(
                    &wgpu_state.renderer.queue,
                    dt,
                    self.time,
                    particle_system.properties.gravity,
                );

                // Dispatch compute shader (GPU handles aging, movement, and death)
                compute_pipeline.dispatch(&mut encoder);

                // Clean up dead particles in CPU-side array for tracking
                particle_system.collect_dead_particles();
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
            // Directional light coming from a lower angle for more visible shadows
            let light_direction = glam::Vec3::new(0.8, -0.5, 0.4).normalize();

            // Calculate actual scene bounds from all entities
            let mut min_bounds = glam::Vec3::splat(f32::MAX);
            let mut max_bounds = glam::Vec3::splat(f32::MIN);
            let mut has_entities = false;

            for entity in scene.entities() {
                let pos = entity.transform.position;
                min_bounds = min_bounds.min(pos);
                max_bounds = max_bounds.max(pos);
                has_entities = true;
            }

            // Fallback if no entities
            if !has_entities {
                min_bounds = glam::Vec3::splat(-10.0);
                max_bounds = glam::Vec3::splat(10.0);
            }

            // Expand bounds slightly for margin
            min_bounds -= glam::Vec3::splat(2.0);
            max_bounds += glam::Vec3::splat(2.0);

            let scene_center = (min_bounds + max_bounds) * 0.5;
            let scene_radius = (max_bounds - min_bounds).length() * 0.5;

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

        // Render skybox first (background) - with MSAA
        if let Some(ref skybox) = wgpu_state.skybox {
            if let Some(ref camera_bind_group) = wgpu_state.camera_bind_group {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Skybox Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &wgpu_state.msaa_texture,
                        resolve_target: Some(&view),
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
                            &wgpu_state.msaa_texture,
                            Some(&view),
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

        // Render foliage (instanced vegetation)
        if let Some(ref mut foliage_renderer) = wgpu_state.foliage_renderer {
            // Collect all foliage instances grouped by vegetation type
            let mut foliage_by_type: std::collections::HashMap<String, Vec<FoliageInstanceGpu>> = std::collections::HashMap::new();

            for entity in scene.entities() {
                if let Some(foliage) = entity.get_component::<Foliage>() {
                    let world_matrix = scene.world_matrix(entity.id);
                    let color_tint = Vec3::from(foliage.color_tint);

                    for instance in &foliage.instances {
                        let local_pos = Vec3::from(instance.position);
                        // Transform local position by entity's world matrix
                        let world_pos = world_matrix.transform_point3(local_pos);

                        let gpu_instance = FoliageInstanceGpu::new(
                            world_pos,
                            instance.rotation_y,
                            instance.scale,
                            color_tint,
                        );

                        foliage_by_type
                            .entry(foliage.vegetation_type.clone())
                            .or_insert_with(Vec::new)
                            .push(gpu_instance);
                    }
                }
            }

            // Update camera uniforms for foliage
            foliage_renderer.update_camera(&wgpu_state.renderer.queue, view_proj, camera.position);

            // Render each vegetation type
            for (veg_type, instances) in &foliage_by_type {
                if instances.is_empty() {
                    continue;
                }

                // Get the mesh for this vegetation type
                if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(veg_type) {
                    if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                        // Update instance buffer
                        foliage_renderer.update_instances(
                            &wgpu_state.renderer.device,
                            &wgpu_state.renderer.queue,
                            instances,
                        );

                        // Create foliage render pass
                        let mut foliage_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Foliage Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &wgpu_state.msaa_texture,
                                resolve_target: Some(&view),
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &wgpu_state.depth_texture,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        foliage_renderer.render(&mut foliage_pass, gpu_mesh, instances.len() as u32);
                    }
                }
            }
        }

        // Render water (transparent, after opaque objects)
        if let Some(ref water_renderer) = wgpu_state.water_renderer {
            for entity in scene.entities() {
                if let Some(water) = entity.get_component::<Water>() {
                    // Update water uniforms with flow per entity
                    water_renderer.update_uniforms(
                        &wgpu_state.renderer.queue,
                        view_proj,
                        camera.position,
                        self.time,
                        water.flow_direction,
                        water.flow_speed,
                    );
                    if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&water.mesh_path) {
                        if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                            let world_matrix = scene.world_matrix(entity.id);

                            // Get water texture
                            let texture_path = water.texture_path.as_deref().unwrap_or("water");
                            let texture_handle = wgpu_state.texture_manager.get_handle(texture_path)
                                .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());
                            let texture_bind_group = &wgpu_state.texture_manager.get_texture(texture_handle)
                                .expect("Texture should exist").bind_group;

                            // Get shadow bind group
                            if let Some(ref shadow_bg) = shadow_sampling_bind_group {
                                // Create water render pass with MSAA
                                let mut water_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Water Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &wgpu_state.msaa_texture,
                                        resolve_target: Some(&view),
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load, // Preserve previous content
                                            store: wgpu::StoreOp::Store,
                                        },
                                        depth_slice: None,
                                    })],
                                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                        view: &wgpu_state.depth_texture,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        }),
                                        stencil_ops: None,
                                    }),
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });

                                water_renderer.render(
                                    &mut water_pass,
                                    gpu_mesh,
                                    world_matrix,
                                    texture_bind_group,
                                    shadow_bg,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Render terrain water bodies (from flood-fill computation)
        if let Some(ref water_renderer) = wgpu_state.water_renderer {
            for water_body in &wgpu_state.terrain_water_bodies {
                // Get the mesh for this water body
                if let Some(mesh_handle) = wgpu_state.mesh_manager.get_handle(&water_body.mesh_name) {
                    if let Some(gpu_mesh) = wgpu_state.mesh_manager.get_mesh(mesh_handle) {
                        // Update water uniforms with this body's flow
                        let flow_dir = water_body.flow_direction.unwrap_or([0.0, 0.0]);
                        water_renderer.update_uniforms(
                            &wgpu_state.renderer.queue,
                            view_proj,
                            camera.position,
                            self.time,
                            flow_dir,
                            water_body.flow_speed,
                        );

                        // Get water texture
                        let texture_handle = wgpu_state.texture_manager.get_handle("water")
                            .unwrap_or_else(|| wgpu_state.texture_manager.white_texture_handle());
                        let texture_bind_group = &wgpu_state.texture_manager.get_texture(texture_handle)
                            .expect("Texture should exist").bind_group;

                        // Get shadow bind group
                        if let Some(ref shadow_bg) = shadow_sampling_bind_group {
                            // Create water render pass with MSAA
                            let mut water_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Terrain Water Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &wgpu_state.msaa_texture,
                                    resolve_target: Some(&view),
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: &wgpu_state.depth_texture,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                }),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            // Terrain water is already in world space (identity transform)
                            water_renderer.render(
                                &mut water_pass,
                                gpu_mesh,
                                glam::Mat4::IDENTITY,
                                texture_bind_group,
                                shadow_bg,
                            );
                        }
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
                    let alive_count = particle_system.particles.iter().filter(|p| p.is_alive()).count();

                    if alive_count > 0 {
                        log::info!("Rendering entity {}: {} alive particles (buffer size: {})",
                                  entity_id.0, alive_count, particle_count);
                    }

                    if particle_count > 0 {
                        // Particle render pass with MSAA
                        let mut particle_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Particle Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &wgpu_state.msaa_texture,
                                resolve_target: Some(&view),
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

        // Render egui UI and capture editor changes
        let (paint_jobs, textures_delta, screen_descriptor, editor_result) = {
            let ui = self.ui.as_mut().unwrap();
            let egui_state = self.egui_state.as_mut().unwrap();
            let can_undo = self.undo_history.can_undo();
            let can_redo = self.undo_history.can_redo();

            let raw_input = egui_state.winit_state.take_egui_input(window);
            let mut editor_result = EditorResult::default();
            let full_output = egui_state.context.run(raw_input, |ctx| {
                editor_result = ui.render(ctx, scene, can_undo, can_redo);
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

            (paint_jobs, full_output.textures_delta, screen_descriptor, editor_result)
        };

        // Handle entity creation from hierarchy panel
        if let Some((entity_name, parent_id)) = editor_result.hierarchy.create_entity {
            self.undo_history.push_state(scene);
            let new_id = scene.create_entity(entity_name);
            if let Some(parent) = parent_id {
                scene.set_parent(new_id, Some(parent));
            }
            if let Some(ui) = self.ui.as_mut() {
                ui.selected_entity = Some(new_id);
                ui.mark_scene_modified();
            }
        }

        // Handle entity deletion from hierarchy panel
        if let Some(entity_id) = editor_result.hierarchy.delete_entity {
            self.undo_history.push_state(scene);
            scene.remove_entity(entity_id);
            if let Some(ui) = self.ui.as_mut() {
                if ui.selected_entity == Some(entity_id) {
                    ui.selected_entity = None;
                }
                ui.mark_scene_modified();
            }
        }

        // Handle entity duplication from hierarchy panel
        if let Some(entity_id) = editor_result.hierarchy.duplicate_entity {
            self.undo_history.push_state(scene);
            if let Some(new_id) = scene.duplicate_entity(entity_id) {
                if let Some(ui) = self.ui.as_mut() {
                    ui.selected_entity = Some(new_id);
                    ui.mark_scene_modified();
                }
            }
        }

        // Handle entity reparenting from hierarchy panel
        if let Some((child_id, new_parent_id)) = editor_result.hierarchy.reparent {
            self.undo_history.push_state(scene);
            scene.set_parent(child_id, new_parent_id);
            if let Some(ui) = self.ui.as_mut() {
                ui.mark_scene_modified();
            }
        }

        // Handle entity renaming from hierarchy panel
        if let Some((entity_id, new_name)) = editor_result.hierarchy.rename_entity {
            self.undo_history.push_state(scene);
            if let Some(entity) = scene.get_entity_mut(entity_id) {
                entity.name = new_name;
            }
            if let Some(ui) = self.ui.as_mut() {
                ui.mark_scene_modified();
            }
        }

        // Handle terrain/water regeneration from inspector changes
        if editor_result.inspector.terrain_changed {
            self.undo_history.push_state(scene);
            wgpu_state.terrain_needs_regeneration = true;
            wgpu_state.water_needs_regeneration = true; // Water depends on terrain
            if let Some(ui) = self.ui.as_mut() {
                ui.mark_scene_modified();
            }
        }
        if editor_result.inspector.water_changed {
            self.undo_history.push_state(scene);
            wgpu_state.water_needs_regeneration = true;
            if let Some(ui) = self.ui.as_mut() {
                ui.mark_scene_modified();
            }
        }

        // Handle component changes from inspector
        if editor_result.inspector.components_changed {
            self.undo_history.push_state(scene);
            if let Some(ui) = self.ui.as_mut() {
                ui.mark_scene_modified();
            }
        }

        // Handle undo/redo requests from Edit menu
        if editor_result.undo_requested {
            if self.undo_history.undo(scene) {
                if let Some(ui) = self.ui.as_mut() {
                    ui.selected_entity = None; // Selection may be invalid after undo
                    ui.mark_scene_modified();
                }
                log::info!("Undo (remaining: {})", self.undo_history.undo_count());
            }
        }
        if editor_result.redo_requested {
            if self.undo_history.redo(scene) {
                if let Some(ui) = self.ui.as_mut() {
                    ui.selected_entity = None; // Selection may be invalid after redo
                    ui.mark_scene_modified();
                }
                log::info!("Redo (remaining: {})", self.undo_history.redo_count());
            }
        }

        // Clear undo history when scene is loaded or new scene created
        if editor_result.scene_changed {
            self.undo_history.clear();
            log::info!("Undo history cleared (scene changed)");
        }

        // Handle opening recent file
        if let Some(path) = editor_result.open_recent_file {
            if std::path::Path::new(&path).exists() {
                match Scene::load_from_file(&path) {
                    Ok(loaded_scene) => {
                        if let Some(scene) = &mut self.scene {
                            *scene = loaded_scene;
                        }
                        if let Some(ui) = &mut self.ui {
                            ui.log_info(format!("Scene loaded from: {}", path));
                            ui.add_recent_file(path.clone());
                            ui.current_scene_path = Some(path);
                            ui.scene_modified = false;
                            ui.selected_entity = None;
                        }
                        self.undo_history.clear();
                        log::info!("Loaded recent file, undo history cleared");
                    }
                    Err(e) => {
                        if let Some(ui) = &mut self.ui {
                            ui.log_error(format!("Failed to load scene: {}", e));
                        }
                    }
                }
            } else {
                if let Some(ui) = &mut self.ui {
                    ui.log_error(format!("File not found: {}", path));
                    // Remove from recent files if it no longer exists
                    ui.recent_files.retain(|p| p != &path);
                }
            }
        }

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
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }
            _ => {}
        }

        // Handle keyboard shortcuts
        if let WindowEvent::KeyboardInput {
            event: KeyEvent {
                state: ElementState::Pressed,
                physical_key: PhysicalKey::Code(key_code),
                ..
            },
            ..
        } = event {
            // Ctrl+D - Duplicate entity
            if self.modifiers.control_key() && key_code == KeyCode::KeyD {
                // Push undo state before duplicating
                if let Some(scene) = &self.scene {
                    self.undo_history.push_state(scene);
                }
                if let (Some(scene), Some(ui)) = (&mut self.scene, &mut self.ui) {
                    if let Some(entity_id) = ui.selected_entity {
                        if let Some(new_id) = scene.duplicate_entity(entity_id) {
                            ui.selected_entity = Some(new_id);
                            ui.mark_scene_modified();
                            log::info!("Duplicated entity {:?} -> {:?}", entity_id, new_id);
                        }
                    }
                }
            }
            // F - Focus camera on selected entity
            if key_code == KeyCode::KeyF && !self.modifiers.control_key() {
                if let (Some(scene), Some(ui)) = (&self.scene, &self.ui) {
                    if let Some(entity_id) = ui.selected_entity {
                        if let Some(entity) = scene.get_entity(entity_id) {
                            // Focus camera on entity position
                            let target = entity.transform.position;
                            self.viewport_controls.pan_offset = target;
                            // Adjust orbit distance based on entity scale
                            let max_scale = entity.transform.scale.x
                                .max(entity.transform.scale.y)
                                .max(entity.transform.scale.z);
                            self.viewport_controls.orbit_distance = (max_scale * 5.0).max(5.0).min(50.0);
                            log::info!("Focused on entity: {}", entity.name);
                        }
                    }
                }
            }
            // Ctrl+C - Copy selected entity
            if self.modifiers.control_key() && key_code == KeyCode::KeyC {
                if let (Some(scene), Some(ui)) = (&self.scene, &self.ui) {
                    if let Some(entity_id) = ui.selected_entity {
                        if let Some(entity) = scene.get_entity(entity_id) {
                            // Serialize the entity to clipboard
                            use engine_scene::scene_data::{SerializedEntity, SerializedComponent};
                            use engine_scene::components::*;

                            let mut components = Vec::new();
                            if let Some(c) = entity.get_component::<MeshRenderer>() {
                                components.push(SerializedComponent::MeshRenderer(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<Camera>() {
                                components.push(SerializedComponent::Camera(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<Light>() {
                                components.push(SerializedComponent::Light(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<ParticleEmitter>() {
                                components.push(SerializedComponent::ParticleEmitter(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<Water>() {
                                components.push(SerializedComponent::Water(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<TerrainWater>() {
                                components.push(SerializedComponent::TerrainWater(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<TerrainGenerator>() {
                                components.push(SerializedComponent::TerrainGenerator(c.clone()));
                            }
                            if let Some(c) = entity.get_component::<Foliage>() {
                                components.push(SerializedComponent::Foliage(c.clone()));
                            }

                            self.clipboard = Some(SerializedEntity {
                                id: entity.id,
                                name: entity.name.clone(),
                                transform: entity.transform,
                                parent: None, // Don't copy parent relationship
                                children: Vec::new(), // Don't copy children
                                components,
                            });
                            log::info!("Copied entity: {}", entity.name);
                        }
                    }
                }
            }
            // Ctrl+V - Paste entity from clipboard
            if self.modifiers.control_key() && key_code == KeyCode::KeyV {
                if let Some(ref clipboard) = self.clipboard.clone() {
                    // Push undo state before pasting
                    if let Some(scene) = &self.scene {
                        self.undo_history.push_state(scene);
                    }
                    if let (Some(scene), Some(ui)) = (&mut self.scene, &mut self.ui) {
                        use engine_scene::scene_data::SerializedComponent;

                        // Create a new entity with offset position
                        let new_name = format!("{} (Copy)", clipboard.name);
                        let new_id = scene.create_entity(new_name);

                        if let Some(entity) = scene.get_entity_mut(new_id) {
                            // Copy transform with offset
                            entity.transform = clipboard.transform;
                            entity.transform.position.x += 1.0; // Offset so it's visible

                            // Add components from clipboard
                            for component in &clipboard.components {
                                match component {
                                    SerializedComponent::MeshRenderer(c) => entity.add_component(c.clone()),
                                    SerializedComponent::Camera(c) => entity.add_component(c.clone()),
                                    SerializedComponent::Light(c) => entity.add_component(c.clone()),
                                    SerializedComponent::ParticleEmitter(c) => entity.add_component(c.clone()),
                                    SerializedComponent::Water(c) => entity.add_component(c.clone()),
                                    SerializedComponent::TerrainWater(c) => entity.add_component(c.clone()),
                                    SerializedComponent::TerrainGenerator(c) => entity.add_component(c.clone()),
                                    SerializedComponent::Foliage(c) => entity.add_component(c.clone()),
                                    SerializedComponent::Generic { .. } => {}
                                }
                            }
                        }

                        ui.selected_entity = Some(new_id);
                        ui.mark_scene_modified();
                        log::info!("Pasted entity: {} (Copy)", clipboard.name);
                    }
                }
            }
            // Delete key - Delete selected entity
            if key_code == KeyCode::Delete {
                if let Some(scene) = &self.scene {
                    self.undo_history.push_state(scene);
                }
                if let (Some(scene), Some(ui)) = (&mut self.scene, &mut self.ui) {
                    if let Some(entity_id) = ui.selected_entity {
                        scene.remove_entity(entity_id);
                        ui.selected_entity = None;
                        ui.mark_scene_modified();
                        log::info!("Deleted entity {:?}", entity_id);
                    }
                }
            }
            // Ctrl+Z - Undo (handles both scene and terrain)
            if self.modifiers.control_key() && key_code == KeyCode::KeyZ && !self.modifiers.shift_key() {
                // Check if we're in terrain mode and have terrain undo states
                let is_terrain_mode = self.ui.as_ref().map(|ui| ui.brush_tool.mode.is_terrain_mode()).unwrap_or(false);

                if is_terrain_mode && !self.terrain_undo_stack.is_empty() {
                    // Terrain undo
                    if let Some(wgpu_state) = &mut self.wgpu_state {
                        if let (Some(ref mut heightmap), Some(ref config)) = (&mut wgpu_state.terrain_heightmap, &wgpu_state.terrain_config) {
                            // Save current state to redo stack
                            self.terrain_redo_stack.push(heightmap.heights.clone());
                            // Restore from undo stack
                            if let Some(old_heights) = self.terrain_undo_stack.pop() {
                                heightmap.heights = old_heights;
                                // Regenerate terrain mesh
                                let terrain_mesh = Terrain::generate_mesh_from_heightmap(heightmap, config);
                                let gpu_vertices = convert_mesh_to_gpu(&terrain_mesh);
                                wgpu_state.mesh_manager.upload_mesh(
                                    &wgpu_state.renderer.device,
                                    "terrain".to_string(),
                                    &gpu_vertices,
                                    &terrain_mesh.indices,
                                );
                                log::info!("Terrain undo (remaining: {})", self.terrain_undo_stack.len());
                            }
                        }
                    }
                } else {
                    // Scene undo
                    if let (Some(scene), Some(ui)) = (&mut self.scene, &mut self.ui) {
                        if self.undo_history.undo(scene) {
                            ui.selected_entity = None;
                            ui.mark_scene_modified();
                            log::info!("Undo (remaining: {})", self.undo_history.undo_count());
                        }
                    }
                }
            }
            // Ctrl+Y or Ctrl+Shift+Z - Redo (handles both scene and terrain)
            if (self.modifiers.control_key() && key_code == KeyCode::KeyY) ||
               (self.modifiers.control_key() && self.modifiers.shift_key() && key_code == KeyCode::KeyZ) {
                // Check if we're in terrain mode and have terrain redo states
                let is_terrain_mode = self.ui.as_ref().map(|ui| ui.brush_tool.mode.is_terrain_mode()).unwrap_or(false);

                if is_terrain_mode && !self.terrain_redo_stack.is_empty() {
                    // Terrain redo
                    if let Some(wgpu_state) = &mut self.wgpu_state {
                        if let (Some(ref mut heightmap), Some(ref config)) = (&mut wgpu_state.terrain_heightmap, &wgpu_state.terrain_config) {
                            // Save current state to undo stack
                            self.terrain_undo_stack.push(heightmap.heights.clone());
                            // Restore from redo stack
                            if let Some(new_heights) = self.terrain_redo_stack.pop() {
                                heightmap.heights = new_heights;
                                // Regenerate terrain mesh
                                let terrain_mesh = Terrain::generate_mesh_from_heightmap(heightmap, config);
                                let gpu_vertices = convert_mesh_to_gpu(&terrain_mesh);
                                wgpu_state.mesh_manager.upload_mesh(
                                    &wgpu_state.renderer.device,
                                    "terrain".to_string(),
                                    &gpu_vertices,
                                    &terrain_mesh.indices,
                                );
                                log::info!("Terrain redo (remaining: {})", self.terrain_redo_stack.len());
                            }
                        }
                    }
                } else {
                    // Scene redo
                    if let (Some(scene), Some(ui)) = (&mut self.scene, &mut self.ui) {
                        if self.undo_history.redo(scene) {
                            ui.selected_entity = None;
                            ui.mark_scene_modified();
                            log::info!("Redo (remaining: {})", self.undo_history.redo_count());
                        }
                    }
                }
            }
            // F1 - Show keyboard shortcuts help
            if key_code == KeyCode::F1 {
                if let Some(ui) = &mut self.ui {
                    ui.show_shortcuts_help = !ui.show_shortcuts_help;
                }
            }
            // F2 - Rename selected entity
            if key_code == KeyCode::F2 {
                if let (Some(scene), Some(ui)) = (&self.scene, &mut self.ui) {
                    if let Some(entity_id) = ui.selected_entity {
                        if let Some(entity) = scene.get_entity(entity_id) {
                            ui.hierarchy_state.editing_entity = Some(entity_id);
                            ui.hierarchy_state.editing_name = entity.name.clone();
                        }
                    }
                }
            }
            // Home - Reset camera view to default position
            if key_code == KeyCode::Home {
                self.viewport_controls.orbit_distance = 15.0;
                self.viewport_controls.orbit_yaw = 0.3;
                self.viewport_controls.orbit_pitch = 0.4;
                self.viewport_controls.pan_offset = glam::Vec3::new(0.0, 5.0, 0.0);
                log::info!("Camera view reset");
            }
            // Escape - Deselect entity or exit
            if key_code == KeyCode::Escape {
                if let Some(ui) = &mut self.ui {
                    if ui.selected_entity.is_some() {
                        // Deselect the entity
                        ui.selected_entity = None;
                        log::info!("Entity deselected");
                    } else if ui.scene_modified {
                        // Show exit confirmation if scene has unsaved changes
                        ui.show_exit_confirm = true;
                    } else {
                        // Exit the editor
                        ui.exit_requested = true;
                    }
                }
            }
        }

        // Handle other window events
        match event {
            WindowEvent::CloseRequested => {
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
