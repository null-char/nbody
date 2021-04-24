use crate::constants::{MAX_X, MAX_Y, MIN_X, MIN_Y};
use crate::primitives::scalar::Scalar;
use cgmath;
use std::default::Default;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadBoundingBox {
    pub min_x: Scalar,
    pub max_x: Scalar,
    pub min_y: Scalar,
    pub max_y: Scalar,
}

impl QuadBoundingBox {
    /// Gets the center of the x axis of the bounding box
    pub fn cx(&self) -> Scalar {
        (self.min_x + self.max_x) / 2.0
    }

    /// Gets the center of the y axis of the bounding box
    pub fn cy(&self) -> Scalar {
        (self.min_y + self.max_y) / 2.0
    }

    /// Gets the length of the bounding box
    pub fn length(&self) -> Scalar {
        self.max_x - self.min_x
    }

    /// If the point does not lie in this bounding box, `contains` will return false
    pub fn contains(&self, p: cgmath::Vector2<Scalar>) -> bool {
        let (px, py) = (p.x, p.y);
        return (px <= self.max_x && py <= self.max_y) && (px >= self.min_x && py >= self.min_y);
    }

    /// Given a point, determine which quadrant it lies in
    pub fn get_point_quadrant(&self, p: cgmath::Vector2<Scalar>) -> usize {
        // 0 if left half. 1 if right half.
        let x_bit = (p.x >= self.cx()) as usize;
        // 0 if top half. 1 if bottom half.
        let y_bit = (p.y <= self.cy()) as usize;

        // y_bit is shifted left (logical left shift). This will have an effect of "doubling"
        // the number (so 1 -> 2 if point p lies in the bottom half)
        return x_bit + (y_bit << 1);
    }

    /// Gets the child bounding box given a quadrant index
    /// Quadrant indices go from 0 -> 3
    /// Indices 0 -> 1 represents left -> right of the top half
    /// Indices 2 -> 3 represents left -> right of the bottom half
    pub fn get_child_bb(&self, quadrant: usize) -> Self {
        match quadrant {
            0 => Self {
                min_x: self.min_x,
                max_x: self.cx(),
                min_y: self.cy(),
                max_y: self.max_y,
            },
            1 => Self {
                min_x: self.cx(),
                max_x: self.max_x,
                min_y: self.cy(),
                max_y: self.max_y,
            },
            2 => Self {
                min_x: self.min_x,
                max_x: self.cx(),
                min_y: self.min_y,
                max_y: self.cy(),
            },
            3 => Self {
                min_x: self.cx(),
                max_x: self.max_x,
                min_y: self.min_y,
                max_y: self.cy(),
            },
            _ => self.clone(),
        }
    }
}

impl Default for QuadBoundingBox {
    fn default() -> Self {
        Self {
            min_x: MIN_X,
            max_x: MAX_X,
            min_y: MIN_Y,
            max_y: MAX_Y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static BB: QuadBoundingBox = QuadBoundingBox {
        min_x: 0.0,
        max_x: 1000.0,
        min_y: 0.0,
        max_y: 1000.0,
    };

    #[test]
    fn it_computes_cx() {
        let cx = BB.cx();
        assert_eq!(cx, 500.0);
    }

    #[test]
    fn it_computes_cy() {
        let cy = BB.cy();
        assert_eq!(cy, 500.0);
    }

    #[test]
    fn it_computes_length() {
        let bb2 = QuadBoundingBox {
            min_x: 400.0,
            max_x: 700.0,
            min_y: 200.0,
            max_y: 500.0,
        };

        assert_eq!(bb2.length(), 300.0);
    }

    #[test]
    fn it_gets_quadrant_given_a_point() {
        let point = cgmath::Vector2::new(327.0, 587.0);
        assert_eq!(BB.get_point_quadrant(point), 0);
        let point = cgmath::Vector2::new(738.0, 587.0);
        assert_eq!(BB.get_point_quadrant(point), 1);
        let point = cgmath::Vector2::new(327.0, 187.0);
        assert_eq!(BB.get_point_quadrant(point), 2);
        let point = cgmath::Vector2::new(960.0, 187.0);
        assert_eq!(BB.get_point_quadrant(point), 3);
    }

    #[test]
    fn it_computes_child_quadrant() {
        assert_eq!(
            BB.get_child_bb(1),
            QuadBoundingBox {
                min_x: 500.0,
                max_x: 1000.0,
                min_y: 500.0,
                max_y: 1000.0
            }
        );
    }

    #[test]
    fn it_checks_if_bb_contains_point() {
        assert_eq!(BB.contains(cgmath::vec2(1200.0, 600.0)), false);
        assert_eq!(BB.contains(cgmath::vec2(0.0, 600.0)), true);
        assert_eq!(BB.contains(cgmath::vec2(600.0, 1200.0)), false);
        assert_eq!(BB.contains(cgmath::vec2(1200.0, 1200.0)), false);
    }
}
