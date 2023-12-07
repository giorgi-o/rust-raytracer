use kd_tree::KdPoint;

use crate::core::{colour::Colour, vector::Vector, vertex::Vertex};

use super::ray::Ray;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PhotonType {
    Colour,
    Shadow,
    Caustic,
    Vueon,
}

pub struct Photon {
    pub position: Vertex,
    pub incident: Vector, // from the light to the photon, normalised
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
            incident: direction,
            intensity,
            photon_type,
        }
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

pub struct InFlightPhoton {
    pub origin: Vertex,
    pub direction: Vector,
    pub intensity: Colour,
    pub photon_type: PhotonType,
}

impl InFlightPhoton {
    pub fn new(
        origin: Vertex,
        direction: Vector,
        intensity: Colour,
        photon_type: PhotonType,
    ) -> Self {
        Self {
            origin,
            direction,
            intensity,
            photon_type,
        }
    }

    pub fn ray(&self) -> Ray {
        Ray::new(self.origin.clone(), self.direction)
    }

    pub fn landed(&self, position: Vertex) -> Photon {
        Photon::new(position, self.direction, self.intensity, self.photon_type)
    }
}

impl From<Photon> for InFlightPhoton {
    fn from(photon: Photon) -> Self {
        Self {
            origin: photon.position,
            direction: photon.incident,
            intensity: photon.intensity,
            photon_type: photon.photon_type,
        }
    }
}
