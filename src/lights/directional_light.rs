use crate::core::{vector::Vector, colour::Colour, vertex::Vertex};

use super::light::Light;


pub struct DirectionalLight {
    pub direction: Vector,
    pub intensity: Colour,
}

impl DirectionalLight {
    pub fn new(direction: Vector, intensity: Colour) -> Self {
        Self {
            direction,
            intensity,
        }
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