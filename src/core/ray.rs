use super::{vertex::Vertex, vector::Vector};

pub struct Ray {
    position: Vertex,
    direction: Vector,
}

impl Ray {
    pub fn new(position: Vertex, direction: Vector) -> Self {
        Self {
            position,
            direction,
        }
    }

    pub fn position(&self) -> &Vertex {
        &self.position
    }

    pub fn direction(&self) -> &Vector {
        &self.direction
    }
}