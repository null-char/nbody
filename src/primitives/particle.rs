use crate::primitives::instance::Instance;
use crate::utils;
use crate::{constants, utils::generate_new_uuid};
use cgmath::num_traits::Pow;
use uuid::Uuid;

// Not too happy about the copy paste of properties but this will have to do
// for now
pub struct ParticleProperties {
    pub position: cgmath::Vector2<f32>,
    pub mass: f32,
    pub radius: f32,
    pub velocity: cgmath::Vector2<f32>,
    pub acceleration: cgmath::Vector2<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub id: Uuid,
    pub position: cgmath::Vector2<f32>,
    pub mass: f32,
    pub radius: f32,
    pub velocity: cgmath::Vector2<f32>,
    pub acceleration: cgmath::Vector2<f32>,
}

impl Particle {
    pub fn empty() -> Self {
        Self {
            id: generate_new_uuid(),
            position: cgmath::vec2(0.0, 0.0),
            mass: 0.0,
            radius: 0.0,
            velocity: cgmath::vec2(0.0, 0.0),
            acceleration: cgmath::vec2(0.0, 0.0),
        }
    }

    pub fn new(properties: ParticleProperties) -> Self {
        Self {
            id: generate_new_uuid(),
            position: properties.position,
            mass: properties.mass,
            radius: properties.radius,
            velocity: properties.velocity,
            acceleration: properties.acceleration,
        }
    }

    pub fn check_collision(&self, p2: &Self) -> bool {
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

    /// Compares two particles and returns them as the tuple (lesser, greater)
    /// where the first element is the particle with the lower mass and the second
    /// element is the particle with the greater mass.
    pub fn compare(self, p2: Self) -> (Self, Self) {
        if self.mass >= p2.mass {
            (p2, self)
        } else {
            (self, p2)
        }
    }

    /// Converts a particle into an `Instance` to be fed into
    /// the instance buffer for the GPU
    pub fn to_instance(self) -> Instance {
        let mut inst = Instance {
            position: [self.position.x, self.position.y],
            radius: self.radius,
        };
        let (x, y) = (inst.position[0], inst.position[1]);
        let ndc = utils::normalize_window_coordinates(&utils::ViewportTransformOptions {
            window_pos: cgmath::Vector2::new(x as f64, y as f64),
            xw: utils::MinMax::<f64> {
                min: constants::MIN_X as f64,
                max: constants::MAX_X as f64,
            },
            yw: utils::MinMax::<f64> {
                min: constants::MIN_Y as f64,
                max: constants::MAX_Y as f64,
            },
            xv: utils::MinMax::<f64> {
                min: -1.0,
                max: 1.0,
            },
            yv: utils::MinMax::<f64> {
                min: -1.0,
                max: 1.0,
            },
        });
        inst.position = [ndc.x, ndc.y];
        inst.radius /= constants::MAX_X / 2.0;

        inst
    }
}
