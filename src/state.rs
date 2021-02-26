use crate::primitives::{
    draw,
    draw::{DrawBuffers, DrawCircleOptions},
    vertex::Vertex,
};
use std::{borrow::Cow, fs};
use wgpu::{
    util::DeviceExt, CommandEncoderDescriptor, DeviceDescriptor, Instance,
    PipelineLayoutDescriptor, RenderPassColorAttachmentDescriptor, RenderPassDescriptor,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, SwapChainDescriptor,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();
        let instance = Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter({
                &RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                }
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let options = shaderc::CompileOptions::new().unwrap();
        let mut compiler = shaderc::Compiler::new().unwrap();

        // vertex shader module
        let vx_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            flags: wgpu::ShaderFlags::default(),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(
                compiler
                    .compile_into_spirv(
                        fs::read_to_string("shader.vert").unwrap().as_str(),
                        shaderc::ShaderKind::Vertex,
                        "shader.vert",
                        "main",
                        Some(&options),
                    )
                    .unwrap()
                    .as_binary(),
            )),
        });
        // fragment shader module
        let fg_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            flags: wgpu::ShaderFlags::default(),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(
                compiler
                    .compile_into_spirv(
                        fs::read_to_string("shader.frag").unwrap().as_str(),
                        shaderc::ShaderKind::Fragment,
                        "shader.frag",
                        "main",
                        Some(&options),
                    )
                    .unwrap()
                    .as_binary(),
            )),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let format = adapter.get_swap_chain_preferred_format(&surface);
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            primitive: wgpu::PrimitiveState::default(),
            fragment: Some(wgpu::FragmentState {
                entry_point: "main",
                module: &fg_module,
                targets: &[wgpu::ColorTargetState {
                    alpha_blend: wgpu::BlendState::default(),
                    color_blend: wgpu::BlendState::default(),
                    write_mask: wgpu::ColorWrite::ALL,
                    format,
                }],
            }),
            vertex: wgpu::VertexState {
                entry_point: "main",
                module: &vx_module,
                buffers: &[wgpu::VertexBufferLayout {
                    step_mode: wgpu::InputStepMode::Vertex,
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    attributes: &[
                        wgpu::VertexAttribute {
                            shader_location: 0,
                            offset: 0,
                            format: wgpu::VertexFormat::Float2,
                        },
                        wgpu::VertexAttribute {
                            shader_location: 1,
                            offset: std::mem::size_of::<[f32; 2]>() as u64,
                            format: wgpu::VertexFormat::Float3,
                        },
                    ],
                }],
            },
        });

        // Creation of the vertex buffer
        let center = cgmath::Vector2::new(0.0, 0.0);
        let radius: f32 = 0.3;
        // Construction of the circle via a triangle fan
        let DrawBuffers { vertices, indices } = draw::create_circle(DrawCircleOptions {
            center,
            color: cgmath::Vector3::new(1.0, 1.0, 1.0),
            radius,
            window_size,
        });
        let indices = indices.unwrap();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsage::INDEX,
        });

        let sc_desc = SwapChainDescriptor {
            present_mode: wgpu::PresentMode::Fifo,
            height: window_size.height,
            width: window_size.width,
            format,
            // RENDER_ATTACHMENT implies that the textures will be used to write to the screen
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size: window_size,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            render_pipeline,
            vertex_buffer,
            num_vertices: vertices.len() as u32,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // Recreate vertex buffer
        let center = cgmath::Vector2::new(0.0, 0.0);
        let radius: f32 = 0.3;
        // Construction of the circle via a triangle fan
        let DrawBuffers { vertices, .. } = draw::create_circle(DrawCircleOptions {
            center,
            color: cgmath::Vector3::new(1.0, 1.0, 1.0),
            radius,
            window_size: new_size,
        });

        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });

        // We'll need to recreate the swap chain on resize events. We'll just mutate
        // the internal state then just recreate the swap chain with the now
        // changed state
        self.size = new_size;
        self.sc_desc.height = new_size.height;
        self.sc_desc.width = new_size.width;
        // Swap chain will be recreated with the new values
        self.recreate_swap_chain();
    }

    pub fn recreate_swap_chain(&mut self) {
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    /// Returns true if an event was captured otherwise this will return false
    pub fn input(&mut self, window_event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame;
        match self.swap_chain.get_current_frame() {
            Ok(sc_frame) => frame = sc_frame.output,
            Err(sc_err) => return Err(sc_err),
        }

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // render pass should be locally scoped so that the mutable borrow to encoder is dropped when we try to `encoder.finish()`
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    // No need to specify target view as the default is `attachment` unless multisampling is enabled
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        let cb = encoder.finish();
        // An iterator that'll just yield once
        self.queue.submit(std::iter::once(cb));
        Ok(())
    }
}
