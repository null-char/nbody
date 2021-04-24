use cgmath::InnerSpace;
use uuid::Uuid;

use crate::quadtree::quadtree::QuadTree;
use crate::{
    primitives::{instance::Instance, particle::Particle, scalar::Scalar},
    quadtree::quadtree::QuadTreeIter,
};

/// Simulation handles all core aspects of simulating the particle system
pub struct Simulation {
    particles: Vec<Particle>,
    time_step: Scalar,
    theta: Scalar,
}

impl Simulation {
    pub fn new(time_step: Scalar, theta: Scalar) -> Self {
        Self {
            particles: Vec::new(),
            time_step,
            theta,
        }
    }

    pub fn step(&mut self) {
        let quadtree: QuadTree = QuadTree::from_points(self.particles.clone());
        let theta = self.theta;

        for p in &mut self.particles {
            let tree_iter = QuadTreeIter::new(p.position, theta, &quadtree);

            for node in tree_iter {
                let node_particle = node.particle;
                let (x, y) = (node_particle.position.x, node_particle.position.y);
                let d = cgmath::vec2(x - p.position.x, y - p.position.y);
                let mass = node_particle.mass;
                p.acceleration = (mass / d.magnitude2()) * d.normalize();
            }
        }
    }

    pub fn resolve_collisions(&mut self) {
        let quadtree: QuadTree = QuadTree::from_points(self.particles.clone());

        // Collision detection using quadtree to figure out a particle's nearby siblings
        for p in self.particles.clone() {
            let mut stack = vec![&quadtree];
            let mut parent = &quadtree;
            let mut nearby_particles = Vec::new();

            // Broad phase (figuring out all the nearby particles to check collision for)
            while !stack.is_empty() {
                let node = stack.pop().unwrap();
                if !node.is_subdivided() && node.particle.id == p.id {
                    let mut p_stack = vec![parent];
                    while !p_stack.is_empty() {
                        let node = p_stack.pop().unwrap();
                        if !node.is_subdivided() && node.particle.id != p.id {
                            nearby_particles.push(node.particle);
                        }

                        for child in &node.children {
                            match child {
                                Some(n) => p_stack.push(n),
                                _ => (),
                            }
                        }
                    }
                }

                if node.is_subdivided() {
                    for child in &node.children {
                        match child {
                            Some(n) => stack.push(n),
                            _ => (),
                        }
                    }
                }
                parent = node;
            }

            // Narrow phase
            for p2 in nearby_particles {
                if p.check_collision(&p2) {
                    self.merge_particle(p, p2);
                }
            }
        }
    }

    fn merge_particle(&mut self, p1: Particle, p2: Particle) {
        let (lesser, greater) = p1.compare(p2);
        let greater_idx = self
            .particles
            .clone()
            .into_iter()
            .position(|p| p.id == greater.id);

        if let Some(idx) = greater_idx {
            let p = self.particles.get_mut(idx).unwrap();
            p.radius += lesser.radius / 10.0;
            p.mass += lesser.mass * p.radius;
            p.velocity =
                (greater.mass * greater.velocity + lesser.mass * lesser.velocity) / greater.mass;

            self.remove_particle(lesser.id);
        }
    }

    /// Removes a particle with the given id.
    fn remove_particle(&mut self, id: Uuid) {
        self.particles = self
            .particles
            .clone()
            .into_iter()
            .filter(|p| p.id != id)
            .collect();
    }

    /// Sums up the forces acting on each particle in the system
    pub fn integrate(&mut self) {
        let particles = &mut self.particles;

        for i in 0..particles.len() {
            let pt = particles.get_mut(i).unwrap();

            let ts = self.time_step;
            pt.velocity += pt.acceleration * ts;
            let position = pt.velocity * ts;
            pt.position += position;

            // Position vector of the vertex closest to the boundary
            let pv = pt.position + pt.velocity.normalize_to(pt.radius);
            if pv.x > 1000.0 || pv.y > 1000.0 || pv.x < 0.0 || pv.y < 0.0 {
                pt.velocity /= 2.0;
                if pt.velocity.magnitude2() < 300.0 {
                    pt.velocity *= 2.0;
                }
                let theta = 3.14 + 0.7;
                let (x, y) = (pt.velocity.x, pt.velocity.y);
                pt.velocity.x = x * f32::cos(theta) - y * f32::sin(theta);
                pt.velocity.y = x * f32::sin(theta) + y * f32::cos(theta);
            }
        }
    }

    pub fn reset(&mut self) {
        self.particles.clear();
    }

    pub fn change_time_step(&mut self, step_offset: Scalar) {
        let new_step = self.time_step + step_offset;
        if new_step > 0.0 {
            self.time_step = new_step;
        }
    }

    /// Adds a particle to the simulation system and also checks for collision
    /// (merges particles if any of them overlap regardless of whether or not
    ///  the simulation is paused)
    pub fn add_particle(&mut self, p: Particle) {
        self.particles.push(p);
        self.resolve_collisions();
    }

    /// Returns a shared reference to particles
    pub fn get_particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    /// Returns a vector containing all the particle instances (copy)
    pub fn get_instances(&self) -> Vec<Instance> {
        self.particles
            .clone()
            .into_iter()
            .map(|p| p.to_instance())
            .collect()
    }
}
