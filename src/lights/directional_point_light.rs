use std::time::Instant;

use rand::seq::SliceRandom;

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
        let direction = self.position.vector_to(surface).normalised();
        let dot = direction.dot(&self.direction);
        Some(self.intensity * dot)
    }

    fn photon_light(self: Box<Self>) -> Box<dyn PhotonLight> {
        self
    }
}

impl PhotonLight for DPLight {
    fn shoot_regular_photons(
        &self,
        scene: &PhotonScene,
        num_photons: u32,
        first_thread: bool,
    ) -> Vec<Photon> {
        let mut photons = Vec::with_capacity(num_photons as usize);
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
            if first_thread && (i % 10000 == 0 || i == num_photons - 1) {
                let progress = (i + 1) as f32 / num_photons as f32;
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

    fn shoot_caustic_photons<'a>(
        &'a self,
        scene: &'a PhotonScene,
        caustic_photons: &[Photon],
        num_photons: u32,
        first_thread: bool,
    ) -> Vec<Photon> {
        let mut photons = Vec::with_capacity(num_photons as usize);
        let mut rng = rand::thread_rng();
        let start = Instant::now();

        for i in 0..num_photons {
            // pick a random existing caustic photon
            let caustic_photon = caustic_photons.choose(&mut rng).unwrap();

            // generate a random offset vector, of length 0.1
            let offset = Vector::random() * 0.1;
            let light_to_photon = self.position.vector_to(&caustic_photon.position);
            let direction = light_to_photon + offset;

            let photon = InFlightPhoton::new(
                self.position.clone(),
                direction.normalised(),
                self.intensity,
                PhotonType::Caustic,
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
