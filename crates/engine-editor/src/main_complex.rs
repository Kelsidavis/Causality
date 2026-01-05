// Game Engine Editor - Main entry point

use anyhow::Result;
use engine_render::{camera::Camera, renderer::Renderer as GpuRenderer};
use glam::Mat4;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

struct EditorApp {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState>,
    egui_state: Option<EguiState>,
    camera: Option<Camera>,
    rotation: f32,
}

struct WgpuState {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    renderer: GpuRenderer,
    depth_texture: wgpu::TextureView,
}

struct EguiState {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EditorApp {
    fn new() -> Self {
        Self {
            window: None,
            wgpu_state: None,
            egui_state: None,
            camera: None,
            rotation: 0.0,
        }
    }

    fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone())?;

        // Initialize renderer
        let renderer = pollster::block_on(GpuRenderer::new(
            &instance,
            &surface,
            size.width,
            size.height,
        ))?;

        // Create depth texture
        let depth_texture = renderer.create_depth_texture(size.width, size.height);

        // Initialize camera
        let camera = Camera::new(size.width, size.height);

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &renderer.device,
            renderer.surface_config.format,
            None,
            1,
            false,
        );

        self.window = Some(window);
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            depth_texture,
        });
        self.egui_state = Some(EguiState {
            ctx: egui_ctx,
            state: egui_state,
            renderer: egui_renderer,
        });
        self.camera = Some(camera);

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let (Some(wgpu_state), Some(camera)) = (&mut self.wgpu_state, &mut self.camera) {
                wgpu_state
                    .renderer
                    .resize(&wgpu_state.surface, new_size.width, new_size.height);
                wgpu_state.depth_texture = wgpu_state
                    .renderer
                    .create_depth_texture(new_size.width, new_size.height);
                camera.update_aspect(new_size.width, new_size.height);
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        let Some(wgpu_state) = &mut self.wgpu_state else {
            return Ok(());
        };
        let Some(egui_state) = &mut self.egui_state else {
            return Ok(());
        };
        let Some(camera) = &self.camera else {
            return Ok(());
        };
        let Some(window) = &self.window else {
            return Ok(());
        };

        // Update rotation
        self.rotation += 0.01;

        // Create model matrix (rotating cube)
        let model = Mat4::from_rotation_y(self.rotation) * Mat4::from_rotation_x(self.rotation * 0.5);

        // Get view projection matrix
        let view_proj = camera.view_projection_matrix();

        // Get current texture for rendering (both 3D and UI)
        let output = wgpu_state.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = wgpu_state.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Render 3D scene
        {
            let uniforms = crate::Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model.to_cols_array_2d(),
            };
            wgpu_state.renderer.queue.write_buffer(&wgpu_state.renderer.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Render Pass"),
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

            render_pass.set_pipeline(&wgpu_state.renderer.render_pipeline);
            render_pass.set_bind_group(0, &wgpu_state.renderer.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, wgpu_state.renderer.vertex_buffer.slice(..));
            render_pass.set_index_buffer(wgpu_state.renderer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..wgpu_state.renderer.num_indices, 0, 0..1);
        }

        // Render egui UI overlay
        let raw_input = egui_state.state.take_egui_input(window);
        let full_output = egui_state.ctx.run(raw_input, |ctx| {
            egui::Window::new("Game Engine Editor")
                .default_pos([10.0, 10.0])
                .default_size([300.0, 200.0])
                .show(ctx, |ui| {
                    ui.heading("Phase 1: Foundation");
                    ui.separator();
                    ui.label("Rendering: wgpu");
                    ui.label(format!("Rotation: {:.2}", self.rotation));
                    ui.separator();
                    ui.label("Camera Position:");
                    ui.label(format!("  x: {:.2}", camera.position.x));
                    ui.label(format!("  y: {:.2}", camera.position.y));
                    ui.label(format!("  z: {:.2}", camera.position.z));
                    ui.separator();
                    if ui.button("Reset Rotation").clicked() {
                        self.rotation = 0.0;
                    }
                });
        });

        egui_state.state.handle_platform_output(window, full_output.platform_output);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [wgpu_state.renderer.surface_config.width, wgpu_state.renderer.surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            egui_state.renderer.update_texture(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, *id, image_delta);
        }

        let clipped_primitives = egui_state.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        egui_state.renderer.update_buffers(&wgpu_state.renderer.device, &wgpu_state.renderer.queue, &mut encoder, &clipped_primitives, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for clipped_primitive in &clipped_primitives {
                egui_state.renderer.render(&mut render_pass, &[clipped_primitive.clone()], &screen_descriptor);
            }
        }

        wgpu_state.renderer.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for id in &full_output.textures_delta.free {
            egui_state.renderer.free_texture(id);
        }

        Ok(())
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Game Engine Editor - Phase 1")
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
        // Handle egui events first
        if let Some(egui_state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let response = egui_state.state.on_window_event(window, &event);
                if response.consumed {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
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
