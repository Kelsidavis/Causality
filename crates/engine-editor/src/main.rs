// Simplified game engine editor - Phase 1: Just the rotating cube

use anyhow::Result;
use engine_render::{camera::Camera, renderer::Renderer};
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
    rotation: f32,
}

struct WgpuState {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    renderer: Renderer,
    depth_texture: wgpu::TextureView,
}

impl EditorApp {
    fn new() -> Self {
        Self {
            window: None,
            wgpu_state: None,
            camera: None,
            rotation: 0.0,
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

        self.window = Some(window);
        self.wgpu_state = Some(WgpuState {
            instance,
            surface,
            renderer,
            depth_texture,
        });
        self.camera = Some(camera);

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

        self.rotation += 0.01;

        let model = glam::Mat4::from_rotation_y(self.rotation) * glam::Mat4::from_rotation_x(self.rotation * 0.5);
        let view_proj = camera.view_projection_matrix();

        wgpu_state.renderer.render(
            &wgpu_state.surface,
            &wgpu_state.depth_texture,
            view_proj,
            model,
        )?;

        Ok(())
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Game Engine Editor - Phase 1: Rotating Cube")
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
