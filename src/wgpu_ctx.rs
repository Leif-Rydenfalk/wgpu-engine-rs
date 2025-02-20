use std::borrow::Cow;
use std::sync::Arc;
use wgpu::MemoryHints::Performance;
use wgpu::{BufferDescriptor, BufferUsages, ShaderSource};
use winit::window::Window;

use crate::Camera;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct InstanceData {
    pub position: [f32; 2],
}

pub struct WgpuCtx<'window> {
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    pub camera: Camera,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl<'window> WgpuCtx<'window> {
    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &surface_config);

        // Create vertex data for triangle
        let radius = 2.0;
        let vertices = [
            Vertex { position: [radius * f32::cos(0.0), radius * f32::sin(0.0)] }, 
            Vertex { position: [radius * f32::cos(2.0 * std::f32::consts::PI / 3.0), radius * f32::sin(2.0 * std::f32::consts::PI / 3.0)] },
            Vertex { position: [radius * f32::cos(4.0 * std::f32::consts::PI / 3.0), radius * f32::sin(4.0 * std::f32::consts::PI / 3.0)] }
        ];

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: std::mem::size_of_val(&vertices) as u64,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: true,
        });

        {
            let mut buffer_view = vertex_buffer.slice(..).get_mapped_range_mut();
            bytemuck::cast_slice_mut::<_, Vertex>(&mut buffer_view).copy_from_slice(&vertices);
        }
        vertex_buffer.unmap();

        // Create instance data for a 10x10 grid
        let mut instances = Vec::new();
        for x in 0..10 {
            for y in 0..10 {
                instances.push(InstanceData {
                    position: [
                        (x as f32 - 4.5) * 0.2, // Center the grid
                        (y as f32 - 4.5) * 0.2,
                    ],
                });
            }
        }

        let num_instances = instances.len() as u32;

        let instance_data_size = std::mem::size_of::<InstanceData>() as u64;
        let buffer_size = instance_data_size * instances.len() as u64;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: buffer_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        let camera = Camera::new();

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        {
            let mut buffer_view = instance_buffer.slice(..).get_mapped_range_mut();
            bytemuck::cast_slice_mut::<_, InstanceData>(&mut buffer_view)
                .copy_from_slice(bytemuck::cast_slice(&instances));
        }
        instance_buffer.unmap();

        let render_pipeline = create_pipeline(&device, surface_config.format, bind_group_layout);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            instance_buffer,
            num_instances,
            uniform_bind_group,
            uniform_buffer,
            camera,
        }
    }

    pub fn update_instances(&mut self, instances: &[InstanceData]) {
        let new_buffer_size = (std::mem::size_of::<InstanceData>() * instances.len()) as u64;
        let current_buffer_size = self.instance_buffer.size();

        // If new data is larger than current buffer, create a new larger buffer
        if new_buffer_size > current_buffer_size {
            // Create new buffer with the new size
            let new_instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: new_buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Replace the old buffer with the new one
            self.instance_buffer = new_instance_buffer;
        }

        // Write the new instance data to the buffer
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        self.num_instances = instances.len() as u32;
    }

    pub fn draw(&mut self) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let view_matrix = self.camera.get_view_matrix();
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&view_matrix));

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            rpass.draw(0..3, 0..self.num_instances);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        self.camera.update_window_size(width as f32, height as f32);
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    bind_group_layout: wgpu::BindGroupLayout
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.txt"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
         bind_group_layouts: &[&bind_group_layout], 
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 1,
                    }],
                },
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
