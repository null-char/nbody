pub struct Particle {
    pub position: cgmath::Vector2<f32>,
    pub mass: f32, // (mass = density * (4/3 * PI * r^3)),
}
