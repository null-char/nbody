use super::utils;
use crate::primitives::{
    draw, draw::DrawBuffers, instance::Instance, particle::Particle, vertex::Vertex,
};
use std::{borrow::Cow, fs};
use wgpu::{
    util::DeviceExt, CommandEncoderDescriptor, DeviceDescriptor, PipelineLayoutDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, SwapChainDescriptor,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::Window,
};

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
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    num_instances: u32,
    particles: Vec<Particle>,
    cursor_pos: PhysicalPosition<f64>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let window_size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
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
                buffers: &[Vertex::desc(), Instance::desc()],
            },
        });

        // Construction of a unit circle via a triangle fan
        let DrawBuffers { vertices, indices } =
            draw::create_unit_circle(cgmath::Vector3::new(1.0, 1.0, 1.0), window_size);
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

        let instances: Vec<Instance> = Vec::new();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
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
            index_buffer,
            num_indices: indices.len() as u32,
            instance_buffer,
            num_instances: instances.len() as u32,
            instances,
            particles: Vec::new(),
            // Keep track of cursor position so that we can later add new
            // particles whenever there's a mouse left click event
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // Reconstruct the unit circle with the new size and recreate the vertex buffer
        let DrawBuffers { vertices, .. } =
            draw::create_unit_circle(cgmath::Vector3::new(1.0, 1.0, 1.0), new_size);

        self.vertex_buffer.destroy();
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
        match window_event {
            // Keep track of cursor position on cursor movement in state
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = *position;
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let winit::event::MouseButton::Left = button {
                    if let winit::event::ElementState::Released = state {
                        let cx = self.cursor_pos.x;
                        let cy = self.cursor_pos.y;
                        let ndc =
                            utils::normalize_window_coordinates(&utils::ViewportTransformOptions {
                                window_pos: cgmath::Vector2::new(cx, cy),
                                xw: utils::MinMax::<f64> {
                                    min: 0.0,
                                    max: self.size.width as f64,
                                },
                                // Min and max needs to be swapped here as the axes in window space begins at
                                // the top left corner and not the bottom left corner.
                                // Since the direction of the y axis is reversed as opposed to the convention, min
                                // and max needs to be swapped
                                yw: utils::MinMax::<f64> {
                                    min: self.size.height as f64,
                                    max: 0.0,
                                },
                            });
                        let radius = 0.05;

                        self.particles.push(Particle {
                            position: cgmath::Vector2::new(ndc.x, ndc.y),
                            radius,
                            mass: 1.0,
                            velocity: cgmath::Vector2::new(0.0, 0.0),
                        });
                        self.instances.push(Instance {
                            position: [ndc.x, ndc.y],
                            radius,
                        });
                        self.num_instances = self.instances.len() as u32;

                        // Destroy and recreate instance buffer with the new instances
                        self.instance_buffer.destroy();
                        self.instance_buffer =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Instance Buffer"),
                                    contents: bytemuck::cast_slice(self.instances.as_slice()),
                                    usage: wgpu::BufferUsage::VERTEX,
                                });
                    }
                }
            }
            _ => return false,
        }
        true
    }

    pub fn update(&mut self) {
        if !self.particles.is_empty() {
            let mut i: usize = 0;
            while i < (self.particles.len() - 1) {
                let p1 = self.particles.get(i).unwrap();
                let mut j = i + 1;
                while j < self.particles.len() {
                    let p2 = self.particles.get(j).unwrap();
                    let does_collide = p1.check_collision(p2);
                    if does_collide {
                        println!("{} collides with {}!", i, j);
                    }
                    j += 1;
                }

                i += 1;
            }
        }
    }

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
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances);
        }

        let cb = encoder.finish();
        // An iterator that'll just yield once
        self.queue.submit(std::iter::once(cb));
        Ok(())
    }
}
