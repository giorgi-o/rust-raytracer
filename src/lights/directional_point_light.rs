use std::time::Instant;

use rand::{distributions::Uniform, Rng};

use crate::{
    core::{
        colour::Colour,
        photon::{InFlightPhoton, Photon, PhotonType},
        vector::Vector,
        vertex::Vertex,
    },
    environments::photon_scene::PhotonScene,
};

use super::light::{Light, PhotonLight};

pub struct DPLight {
    position: Vertex,
    direction: Vector,
    intensity: Colour,
}

impl DPLight {
    pub fn new(position: Vertex, direction: Vector, intensity: Colour) -> Box<Self> {
        Box::new(Self {
            position,
            direction,
            intensity,
        })
    }
}

impl Light for DPLight {
    fn get_direction(&self, surface: &Vertex) -> Option<Vector> {
        let direction = self.position.vector_to(surface);

        if direction.dot(&self.direction) < 0.0 {
            // angle between surface and light is greater than 90 degrees
            return None;
        }

        Some(direction.normalised())
    }

    fn get_intensity(&self, surface: &Vertex) -> Option<Colour> {
        // intensity decreases with angle
        let direction = self.position.vector_to(surface);
        let dot = direction.dot(&self.direction);
        Some(self.intensity * dot)
    }

    fn photon_light(self: Box<Self>) -> Box<dyn PhotonLight> {
        self
    }
}

impl PhotonLight for DPLight {
    fn shoot_photons<'a>(
        &'a self,
        scene: &PhotonScene,
        num_photons: u32,
        first_thread: bool,
    ) -> Vec<Photon> {
        let mut photons = Vec::with_capacity(num_photons as usize);

        let mut rng = rand::thread_rng();
        let distribution = Uniform::from(-1.0..1.0);

        let start = Instant::now();

        for i in 0..num_photons {
            let direction = Vector::random_on_surface(self.direction);

            let photon = InFlightPhoton::new(
                self.position.clone(),
                direction.normalised(),
                self.intensity,
                PhotonType::Colour,
            );

            let traced_photons = scene.photontrace(photon);
            photons.extend(traced_photons);

            // print progress/ETA
            if first_thread && i % 10000 == 0 {
                let progress = i as f32 / num_photons as f32;
                let elapsed = start.elapsed().as_secs_f32();
                let eta = elapsed / progress - elapsed;
                let percent = (progress * 100.0) as u32;
                print!("{percent}% photons shot, elapsed {elapsed:.2}s, ETA {eta:.2}s\t\r");
            }
        }

        if first_thread {
            println!();
        }

        photons
    }
}
