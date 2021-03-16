use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
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
        }
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}
