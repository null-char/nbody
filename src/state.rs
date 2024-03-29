use crate::{
    constants,
    primitives::{draw, draw::DrawBuffers, instance::Instance, particle::Particle, vertex::Vertex},
    simulation::Simulation,
};
use crate::{primitives::particle::ParticleProperties, utils};
use futures::executor::{LocalPool, LocalSpawner};
use futures::task::SpawnExt;
use rand::Rng;
use std::borrow::Cow;
use wgpu::{
    util::DeviceExt, CommandEncoderDescriptor, DeviceDescriptor, PipelineLayoutDescriptor,
    RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, SwapChainDescriptor,
};
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
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
    num_instances: u32,
    cursor_pos: PhysicalPosition<f64>,
    sim: Simulation,
    glyph_brush: GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: LocalPool,
    local_spawner: LocalSpawner,
    /// Whether or not the simulation is paused
    paused: bool,
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
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = LocalPool::new();
        let local_spawner = local_pool.spawner();

        let options = shaderc::CompileOptions::new().unwrap();
        let mut compiler = shaderc::Compiler::new().unwrap();

        // vertex shader module
        let vx_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            flags: wgpu::ShaderFlags::default(),
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(
                compiler
                    .compile_into_spirv(
                        include_str!("shaders/shader.vert"),
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
                        include_str!("shaders/shader.frag"),
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
        let font =
            ab_glyph::FontArc::try_from_slice(include_bytes!("font/Hack-Regular.ttf")).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, format);

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
            // Keep track of cursor position so that we can later add new
            // particles whenever there's a mouse left click event
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            sim: Simulation::new(0.05, 1.0),
            glyph_brush,
            staging_belt,
            local_pool,
            local_spawner,
            paused: true,
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
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                ..
            } => {
                let keycode = input.virtual_keycode;
                if input.state == winit::event::ElementState::Pressed && keycode.is_some() {
                    let kc = keycode.unwrap();
                    let step_offset = 0.05;

                    match kc {
                        winit::event::VirtualKeyCode::Space => {
                            self.paused = !self.paused;
                        }
                        winit::event::VirtualKeyCode::Up => {
                            self.sim.change_time_step(step_offset);
                        }
                        winit::event::VirtualKeyCode::Down => {
                            self.sim.change_time_step(-step_offset);
                        }
                        winit::event::VirtualKeyCode::R => {
                            self.sim.reset();
                            self.recreate_instance_buffer();
                        }
                        _ => (),
                    }
                }
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
                                xv: utils::MinMax::<f64> {
                                    min: constants::MIN_X as f64,
                                    max: constants::MAX_X as f64,
                                },
                                yv: utils::MinMax::<f64> {
                                    min: constants::MIN_Y as f64,
                                    max: constants::MAX_Y as f64,
                                },
                            });

                        let mut rng = rand::thread_rng();
                        let radius = rng.gen_range(1..4) as f32;
                        self.sim.add_particle(Particle::new(ParticleProperties {
                            position: cgmath::vec2(ndc.x, ndc.y),
                            radius,
                            mass: 50.0 * radius,
                            velocity: cgmath::vec2(0.0, 0.0),
                            acceleration: cgmath::vec2(0.0, 0.0),
                        }));

                        self.recreate_instance_buffer();
                    }
                }
            }
            _ => return false,
        }
        true
    }

    pub fn update(&mut self) {
        if !self.paused && !self.sim.get_particles().is_empty() {
            // As long as the simulation isn't paused and we have particles in
            // the system, check for collision, step the simulation, integrate the forces
            // for each body and finally recreate the instance buffer.
            self.sim.resolve_collisions();
            self.sim.step();
            self.sim.integrate();
            self.recreate_instance_buffer();
        }
    }

    /// Destroys the existing instance buffer and recreates it with
    /// current instances. This function must be called each time the
    /// data within instances change.
    fn recreate_instance_buffer(&mut self) {
        let instances = self.sim.get_instances();
        self.instance_buffer.destroy();
        self.instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instances.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });
        self.num_instances = instances.len() as u32;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame;
        match self.swap_chain.get_current_frame() {
            Ok(sc_frame) => { frame = sc_frame.output },
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

        let dt = self.sim.get_time_step();
        self.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![Text::new(format!("time_step: {:.2}", dt).as_str())
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(25.0)],
            ..Section::default()
        });
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &frame.view,
                self.size.width,
                self.size.height,
            )
            .expect("queue draw");

        self.staging_belt.finish();
        let cb = encoder.finish();
        // An iterator that'll just yield once
        self.queue.submit(std::iter::once(cb));
        // Recall unused buffers after finishing
        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt buffers");
        // Run tasks until we encounter a future on which no more progress can be made
        self.local_pool.run_until_stalled();

        Ok(())
    }
}
