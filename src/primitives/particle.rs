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

        let dist = f32::sqrt(f32::pow(x2 - x1, 2) + f32::pow(y2 - y1, 2));
        let does_intersect = {
            let gap = dist - (self.radius + p2.radius);

            if gap <= 0.0 {
                true
            } else {
                false
            }
        };

        return does_intersect;
    }
}
