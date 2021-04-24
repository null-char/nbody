use crate::primitives::{particle::Particle, scalar::Scalar};
use crate::quadtree::bounding_box::QuadBoundingBox;

#[derive(Debug)]
pub struct QuadTree {
    pub bounding_box: QuadBoundingBox,
    pub particle: Particle,
    pub children: Vec<Option<Self>>,
}

impl QuadTree {
    pub fn empty() -> Self {
        Self {
            particle: Particle::empty(),
            bounding_box: QuadBoundingBox::default(),
            children: vec![None, None, None, None],
        }
    }

    pub fn from_points(points: Vec<Particle>) -> Self {
        let mut qt = Self::empty();
        for p in points {
            qt.insert_particle(p);
        }

        qt
    }

    /// Update center of mass.
    /// See https://en.wikipedia.org/wiki/Center_of_mass#A_system_of_particles
    pub fn update_cm(&mut self, x: Scalar, y: Scalar, m: Scalar) {
        let p = &mut self.particle;
        let total_mass = p.mass + m;
        p.position.x = (p.mass * p.position.x + x * m) / total_mass;
        p.position.y = (p.mass * p.position.y + y * m) / total_mass;
        p.mass = total_mass;
    }

    /// Adds a new child to the quad tree. Quadrant and child bounding box is determined based on x and y.
    pub fn add_child(&mut self, particle: Particle) {
        let quadrant = self.bounding_box.get_point_quadrant(particle.position);
        self.children[quadrant] = Some(Self {
            particle,
            bounding_box: self.bounding_box.get_child_bb(quadrant),
            children: vec![None, None, None, None],
        });
    }

    // is_subdivided checks to see if the current quadtree has any child nodes. If it does, then it is already
    // subdivided. If it doesn't, then it needs to be subdivided.
    pub fn is_subdivided(&self) -> bool {
        for child in self.children.as_slice() {
            if child.is_some() {
                return true;
            }
        }
        return false;
    }

    pub fn insert_particle(&mut self, particle: Particle) {
        // In case we get a point that does not lie in our boundary
        if !self.bounding_box.contains(particle.position) {
            return;
        }
        let p = &mut self.particle;
        if p.mass == 0.0 {
            *p = particle;
            return;
        }

        let mut parent = self;
        let mut bounding_box = parent.bounding_box;
        let mut quadrant = bounding_box.get_point_quadrant(particle.position);

        while let Some(_) = &mut parent.children[quadrant] {
            // First, update the center of mass of the parent quadtree
            parent.update_cm(particle.position.x, particle.position.y, particle.mass);
            // Assign child as parent then update the bounding box and quadrant
            parent = parent.children[quadrant].as_mut().unwrap();
            bounding_box = parent.bounding_box;
            quadrant = bounding_box.get_point_quadrant(particle.position);
        }

        // We're on a node that has had no subdivisions. However, this node already has a body inserted into it which means it is a
        // leaf node. Each section must contain at most 1 body, hence we have to subdivide such that this invariant holds true.
        if !parent.is_subdivided() {
            // We're going to subdivide until the particle currently in this node and the particle to be inserted are in different
            // sections.

            // Parent properties before center of mass is updated. This data needs to be used
            // when we reinsert the parent after the subdivison is complete.
            let parent_particle = parent.particle;
            let (_x, _y, _m) = (
                parent_particle.position.x,
                parent_particle.position.y,
                parent_particle.mass,
            );
            parent.update_cm(particle.position.x, particle.position.y, particle.mass);
            // Parent properties after center of mass is updated
            let cm_parent_particle = parent.particle;
            let bb = parent.bounding_box;
            let mut pq = bb.get_point_quadrant(parent_particle.position);

            // While point quadrant and parent quadrant are the same, we keep subdividing until the sections are small enough
            // such that they separate
            while quadrant == pq {
                parent.add_child(cm_parent_particle);
                parent = parent.children[quadrant].as_mut().unwrap();
                pq = parent
                    .bounding_box
                    .get_point_quadrant(parent_particle.position);
                quadrant = parent.bounding_box.get_point_quadrant(particle.position);
            }

            parent.add_child(parent_particle);
        }

        // We have reached our desired cell. Add a new subcell with this point.
        parent.add_child(particle);
    }
}

pub struct QuadTreeIter<'a> {
    /// The point for which net force is being calculated
    p: cgmath::Vector2<Scalar>,
    /// Accuracy metric. If theta is zero, then this degenerates into a brute force sum (quadratic complexity)
    theta: Scalar,
    stack: Vec<&'a QuadTree>,
}

impl<'a> QuadTreeIter<'a> {
    pub fn new(p: cgmath::Vector2<Scalar>, theta: Scalar, root: &'a QuadTree) -> Self {
        Self {
            p,
            theta,
            stack: vec![root],
        }
    }
}

impl<'a> Iterator for QuadTreeIter<'a> {
    type Item = &'a QuadTree;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() {
            let node = self.stack.pop().unwrap();
            let (x, y) = (node.particle.position.x, node.particle.position.y);
            let s = node.bounding_box.length();
            let d = f32::sqrt(f32::powi(x - self.p.x, 2) + f32::powi(y - self.p.y, 2));
            if d == 0.0 {
                continue;
            }

            // If node is a leaf node or if the distance ratio between point and node is low enough
            // (lower the distance ratio, the farther away the two points are in space), then approximate
            // that particle by returning the internal node
            if !node.is_subdivided() || (s / d) < self.theta {
                return Some(node);
            }

            // If node is not sufficiently far away (i.e s/d >= θ), then recurse into
            // the node's children
            for child in &node.children {
                if let Some(node) = child {
                    self.stack.push(node);
                }
            }
        }
        None
    }
}
