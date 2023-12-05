use kd_tree::KdPoint;

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

impl KdPoint for Photon {
    type Scalar = f32;
    type Dim = typenum::U3;

    fn at(&self, i: usize) -> Self::Scalar {
        match i {
            0 => self.position.x,
            1 => self.position.y,
            2 => self.position.z,
            _ => unreachable!(),
        }
    }
}
