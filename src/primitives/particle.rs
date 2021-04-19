use cgmath::num_traits::Pow;

#[derive(Debug)]
pub struct Particle {
    pub position: cgmath::Vector2<f32>,
    pub mass: f32, // (mass = density * (4/3 * PI * r^3)),
    pub radius: f32,
    pub velocity: cgmath::Vector2<f32>,
}

impl Particle {
    pub fn check_collision(&self, p2: &Particle) -> bool {
        let x1 = self.position.x;
        let x2 = p2.position.x;
        let y1 = self.position.y;
        let y2 = p2.position.y;

        // Applying a square root and then comparing with radii_sum is
        // slightly more expensive
        let dist = f32::pow(x2 - x1, 2) + f32::pow(y2 - y1, 2);
        let radii_sum = f32::pow(self.radius + p2.radius, 2);
        return dist <= radii_sum;
    }
}
