use bytemuck::{Pod, Zeroable};
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Instance {
    /// Denotes the center of the circle instance
    pub position: [f32; 2],
    pub radius: f32,
}

impl Instance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 2,
                    offset: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    shader_location: 3,
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float,
                },
            ],
        }
    }
}

unsafe impl Pod for Instance {}
unsafe impl Zeroable for Instance {}
