use crate::core::{colour::Colour, vector::Vector, vertex::Vertex};

use super::light::Light;

pub struct PointLight {
    position: Vertex,
    intensity: Colour,
}

impl PointLight {
    pub fn new(position: Vertex, intensity: Colour) -> Box<Self> {
        Box::new(Self {
            position,
            intensity,
        })
    }
}

impl Light for PointLight {
    fn get_direction(&self, surface: &Vertex) -> Option<Vector> {
        let direction = self.position.vector_to(surface);
        Some(direction.normalised())
    }

    fn get_intensity(&self, _surface: &Vertex) -> Option<Colour> {
        Some(self.intensity)
    }
}
