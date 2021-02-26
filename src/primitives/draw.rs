use crate::primitives::vertex::Vertex;
use cgmath;
use std::f32::consts::PI;
use winit::dpi::PhysicalSize;

pub struct DrawBuffers {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u16>>,
}

pub struct DrawCircleOptions {
    pub radius: f32,
    pub center: cgmath::Vector2<f32>,
    pub color: cgmath::Vector3<f32>,
    pub window_size: PhysicalSize<u32>,
}

/// Creates vertices and indices describing the configured circle. The first value of the tuple
/// are the vertices and the second value are the indices
pub fn create_circle(options: DrawCircleOptions) -> DrawBuffers {
    let DrawCircleOptions {
        radius,
        center,
        color,
        window_size,
    } = options;
    let color: [f32; 3] = [color.x, color.y, color.z];
    let wx = window_size.width as f32;
    let wy = window_size.height as f32;
    let aspect_ratio = wx / wy;

    // 360 vertices circumscribing the circle.
    // One center vertex.
    let num_vertices = 361;
    let mut vbuf = Vec::with_capacity(num_vertices);
    let mut ibuf: Vec<u16> = vec![0, 360, 1];

    vbuf.push(Vertex {
        position: [center.x, center.y, 0.0],
        color,
    });
    for i in 0..(num_vertices - 1) {
        let rad = cgmath::Rad((i as f32) * (PI / 180.0));
        let x_comp = radius * cgmath::Angle::cos(rad);
        let y_comp = radius * cgmath::Angle::sin(rad);
        let pos = cgmath::Vector2::new(x_comp, y_comp * aspect_ratio) + center;
        vbuf.push(Vertex {
            position: [pos.x, pos.y, 0.0],
            color,
        });
    }

    // Connect each vertex v[i] with v[0] (center vertex) and v[i-1] which produces
    // a fan of triangles resulting in a 2D circle
    for i in 1..num_vertices as u16 {
        ibuf.push(0);
        ibuf.push(i);
        ibuf.push(i + 1);
    }

    return DrawBuffers {
        vertices: vbuf,
        indices: Some(ibuf),
    };
}
