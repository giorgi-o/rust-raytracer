use super::{vertex::Vertex, vector::Vector};

pub struct Ray {
    pub position: Vertex,
    pub direction: Vector,
}

impl Ray {
    pub const fn new(position: Vertex, direction: Vector) -> Self {
        Self {
            position,
            direction,
        }
    }
}