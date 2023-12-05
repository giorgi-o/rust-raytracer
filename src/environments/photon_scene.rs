use std::io::Write;

use rand::seq::SliceRandom;

use crate::{
    core::{
        colour::Colour,
        hit::Hit,
        photon::{Photon, PhotonType},
        photon_tree::PhotonTree,
        ray::Ray,
    },
    lights::light::{Light, PhotonLight},
    materials::material::{PhotonBehaviour, PhotonMaterial},
    objects::object::Object,
};

use super::environment::{Environment, RaytraceResult};

const PHOTONS_PER_LIGHT: usize = 100_000_000;

pub struct PhotonScene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn PhotonLight>>,
    photon_map: Option<PhotonTree>,
}

impl PhotonScene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            photon_map: None,
        }
    }

    pub fn photontrace(&self, photon: Photon) -> Option<Photon> {
        let ray = photon.ray();
        let Some(hit) = self.trace(&ray) else {
            return None;
        };

        let material = hit.material.photon_mapped();

        // pick absorb, diffuse or specular based on weights
        let mut rng = rand::thread_rng();
        let choice = [
            PhotonBehaviour::Absorb,
            PhotonBehaviour::Diffuse,
            PhotonBehaviour::Specular,
        ]
        .choose_weighted(&mut rng, |item| material.behaviour_weight(item))
        .unwrap();

        // None // for now
        Some(self.absorb_photon(photon, &hit, material))
    }

    fn absorb_photon(&self, photon: Photon, hit: &Hit, material: &dyn PhotonMaterial) -> Photon {
        // store photon in kd tree
        let photon_intensity = material.compute_photon(self, hit, &photon.direction);
        Photon::new(
            hit.position.clone(),
            photon.direction,
            photon_intensity,
            PhotonType::Diffuse,
        )
    }


}

impl Environment for PhotonScene {
    fn add_object(&mut self, object: Box<dyn Object + 'static>) {
        self.objects.push(object);
    }

    fn add_light(&mut self, light: Box<dyn Light + 'static>) {
        let light = light.photon_light();
        self.lights.push(light);
    }

    fn pre_render(&mut self) {
        let mut photons = Vec::new();
        let lights = std::mem::take(&mut self.lights);
        for light in lights.iter() {
            let light_photons = light.shoot_photons_mt(self, PHOTONS_PER_LIGHT as u32);
            println!("Light emitted {} photons", light_photons.len());
            photons.push(light_photons);
        }
        self.lights = lights;

        // print!("Flattening photon vec... (1/2)\r");
        // let photons: Vec<Vec<Photon>> = photons.into_iter().flatten().collect();
        // println!("Flattening photon vec... (2/2)");
        // let photons: Vec<Photon> = photons.into_iter().flatten().collect();

        let mut flat_photons = Vec::new();
        let mut stdout = std::io::stdout().lock();
        let mut i = 0;
        let max_i = photons.len();
        while let Some(mut vec) = photons.pop() {
            i += 1;
            let mut j = 0;
            let max_j = vec.len();

            while let Some(vec) = vec.pop() {
                j += 1;
                // print!("Flattening photon vec... ({i}/{max_i} {j}/{max_j})\r");
                write!(
                    stdout,
                    "Flattening photon vec... ({i}/{max_i} {j}/{max_j})\r"
                )
                .unwrap();
                stdout.flush().unwrap();

                flat_photons.extend(vec);
            }
        }
        flat_photons.shrink_to_fit();

        println!("\nBuilding photon KD tree...");
        self.photon_map = Some(PhotonTree::build(flat_photons));
    }

    fn raytrace(&self, ray: &Ray) -> RaytraceResult {
        let Some(hit) = self.trace(ray) else {
            return RaytraceResult {
                colour: Colour::black(),
                depth: 0.0,
            };
        };

        // let neighbour_photons = self.photon_map.find_nearest(&hit.position, 50);
        let neighbour_photons = self
            .photon_map
            .as_ref()
            .expect("Photon map not built")
            .get_n_within_radius(&hit.position, 0.1, 5);
        let neighbour_photons_len = neighbour_photons.len();
        if neighbour_photons_len == 0 {
            return RaytraceResult {
                colour: Colour::black(),
                depth: 0.0,
            };
        }

        let furthest_photon_sqrd_distance = neighbour_photons
            .iter()
            .max_by(|a, b| a.squared_distance.partial_cmp(&b.squared_distance).unwrap())
            .unwrap()
            .squared_distance;

        let colour: Colour = neighbour_photons
            .into_iter()
            .fold(Colour::black(), |acc, item_and_distance| {
                acc + item_and_distance.item.intensity
            })
            / (neighbour_photons_len as f32);

        RaytraceResult {
            colour,
            depth: hit.distance,
        }
    }

    fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }
}
