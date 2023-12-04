use rand::{distributions::Uniform, Rng};

use crate::{
    core::{
        colour::Colour,
        photon::{Photon, PhotonType},
        vector::Vector,
        vertex::Vertex,
    },
    environments::photon_scene::PhotonScene,
};

use super::light::{Light, PhotonLight};

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

impl PhotonLight for PointLight {
    fn shoot_photons(&self, scene: &PhotonScene, num_photons: u32) {
        let mut rng = rand::thread_rng();
        let distribution = Uniform::from(-1.0..1.0);

        for _ in 0..num_photons {
            let direction = loop {
                let direction = Vector::new(
                    rng.sample(distribution),
                    rng.sample(distribution),
                    rng.sample(distribution),
                );
                if direction.length() <= 1.0 {
                    break direction;
                }
            };

            let photon = Photon::new(
                self.position.clone(),
                direction,
                self.intensity,
                PhotonType::Diffuse,
            );

            scene.photontrace(photon);
        }
    }
}
