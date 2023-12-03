use crate::core::{colour::Colour, vector::Vector, vertex::Vertex};

use super::light::Light;

pub struct DirectionalLight {
    pub direction: Vector,
    pub intensity: Colour,
}

impl DirectionalLight {
    pub fn new(direction: Vector, intensity: Colour) -> Box<Self> {
        Box::new(Self {
            direction,
            intensity,
        })
    }
}

impl Light for DirectionalLight {
    fn get_direction(&self, _surface: &Vertex) -> Option<Vector> {
        Some(self.direction)
    }

    fn get_intensity(&self, _surface: &Vertex) -> Option<Colour> {
        Some(self.intensity)
    }
}
