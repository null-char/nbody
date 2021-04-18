use crate::constants::QUADTREE_CAPACITY;
use crate::primitives::scalar::Scalar;
use crate::quadtree::bounding_box::QuadBoundingBox;

#[derive(Debug)]
pub struct QuadTree {
    pub x: Scalar,
    pub y: Scalar,
    pub bounding_box: QuadBoundingBox,
    pub mass: Scalar,
    pub children: Vec<Option<Self>>,
    capacity: usize,
}

impl QuadTree {
    pub fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            mass: 0.0,
            bounding_box: QuadBoundingBox::default(),
            children: vec![None, None, None, None],
            // Default capacity is 2
            capacity: QUADTREE_CAPACITY,
        }
    }

    /// Update center of mass.
    /// See https://en.wikipedia.org/wiki/Center_of_mass#A_system_of_particles
    pub fn update_cm(&mut self, x: Scalar, y: Scalar, m: Scalar) {
        let total_mass = self.mass + m;
        self.x = (self.mass * self.x + x * m) / total_mass;
        self.y = (self.mass * self.y + y * m) / total_mass;
        self.mass = total_mass;
    }

    /// Adds a new child to the quad tree. Quadrant and child bounding box is determined based on x and y.
    pub fn add_child(&mut self, x: Scalar, y: Scalar, m: Scalar) {
        let quadrant = self
            .bounding_box
            .get_point_quadrant(cgmath::Vector2::new(x, y));
        self.children[quadrant] = Some(Self {
            x,
            y,
            mass: m,
            bounding_box: self.bounding_box.get_child_bb(quadrant),
            children: vec![None, None, None, None],
            capacity: QUADTREE_CAPACITY,
        });
    }
}
