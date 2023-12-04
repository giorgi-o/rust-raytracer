use crate::core::{colour::Colour, vector::Vector, vertex::Vertex};

use super::ray::Ray;

pub enum PhotonType {
    Diffuse,
    Specular,
    Caustic,
}

pub struct Photon {
    pub position: Vertex,
    pub direction: Vector,
    pub intensity: Colour,
    pub photon_type: PhotonType,
}

impl Photon {
    pub fn new(
        position: Vertex,
        direction: Vector,
        intensity: Colour,
        photon_type: PhotonType,
    ) -> Self {
        Self {
            position,
            direction,
            intensity,
            photon_type,
        }
    }

    pub fn ray(&self) -> Ray {
        Ray::new(self.position.clone(), self.direction)
    }
}
