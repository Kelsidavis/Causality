// GPU Compute Pipeline for Particle Simulation

use crate::particle::GpuParticle;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::util::DeviceExt;

/// Simulation uniforms passed to compute shader
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SimulationUniforms {
    pub delta_time: f32,
    pub time: f32,
    pub _padding1: [f32; 2],  // Align to 16 bytes
    pub gravity: [f32; 3],
    pub _padding2: f32,        // Align gravity to 16 bytes
}

/// GPU compute pipeline for particle simulation
pub struct ParticleComputePipeline {
    /// Compute pipeline
    pipeline: wgpu::ComputePipeline,

    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,

    /// Particle buffer (storage buffer)
    particle_buffer: wgpu::Buffer,

    /// Uniform buffer for simulation parameters
    uniform_buffer: wgpu::Buffer,

    /// Bind group
    bind_group: wgpu::BindGroup,

    /// Number of particles
    particle_count: u32,
}

impl ParticleComputePipeline {
    /// Create a new particle compute pipeline
    pub fn new(
        device: &wgpu::Device,
        particle_count: u32,
        initial_particles: &[GpuParticle],
    ) -> Result<Self> {
        // Create particle storage buffer with space for particle_count
        // If initial_particles is empty, create dead particles to fill the buffer
        let particles_to_upload: Vec<GpuParticle> = if initial_particles.is_empty() {
            // Create dead particles (position.y = -9999.0)
            vec![GpuParticle {
                position: [0.0, -9999.0, 0.0],
                _padding1: 0.0,
                velocity: [0.0, 0.0, 0.0],
                _padding2: 0.0,
                color: [0.0, 0.0, 0.0, 0.0],
                size: 0.0,
                lifetime: 0.0,
                max_lifetime: 0.0,
                rotation: 0.0,
            }; particle_count as usize]
        } else {
            initial_particles.to_vec()
        };

        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Storage Buffer"),
            contents: bytemuck::cast_slice(&particles_to_upload),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::VERTEX,  // For instanced rendering
        });

        // Create uniform buffer
        let uniforms = SimulationUniforms {
            delta_time: 0.0,
            time: 0.0,
            _padding1: [0.0; 2],
            gravity: [0.0, -9.81, 0.0],
            _padding2: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Simulation Uniforms"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Compute Bind Group Layout"),
            entries: &[
                // Particle storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Simulation uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Load compute shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Update Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../engine-render/src/shaders/particle_update.wgsl").into(),
            ),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Update Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            pipeline,
            bind_group_layout,
            particle_buffer,
            uniform_buffer,
            bind_group,
            particle_count,
        })
    }

    /// Update simulation uniforms
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        delta_time: f32,
        time: f32,
        gravity: Vec3,
    ) {
        let uniforms = SimulationUniforms {
            delta_time,
            time,
            _padding1: [0.0; 2],
            gravity: gravity.to_array(),
            _padding2: 0.0,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Upload new particles to GPU
    pub fn upload_particles(&self, queue: &wgpu::Queue, particles: &[GpuParticle]) {
        queue.write_buffer(&self.particle_buffer, 0, bytemuck::cast_slice(particles));
    }

    /// Dispatch compute shader to update particles
    pub fn dispatch(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Update Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        // Calculate workgroup count
        // Workgroup size is 256 (from shader), so we need ceiling(particle_count / 256)
        let workgroup_count = (self.particle_count + 255) / 256;

        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    /// Get particle buffer for rendering
    pub fn particle_buffer(&self) -> &wgpu::Buffer {
        &self.particle_buffer
    }

    /// Get particle count
    pub fn particle_count(&self) -> u32 {
        self.particle_count
    }
}
