use crate::{
    core::{
        colour::Colour,
        hit::{Hit, HitVec},
        photon::{Photon, PhotonType},
        photon_tree::PhotonTree,
        ray::Ray,
        vector::Vector, vertex::Vertex,
    },
    lights::light::{Light, PhotonLight},
    objects::object::Object,
};

use super::environment::{Environment, RaytraceResult};

pub struct PhotonScene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn PhotonLight>>,
    photon_map: PhotonTree,
}

impl PhotonScene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            photon_map: PhotonTree::new(),
        }
    }

    pub fn photontrace(&self, incoming_photon: Photon) {
        let ray = incoming_photon.ray();
        let Some(hit) = self.trace(&ray) else {
            return;
        };

        let material = hit.material.photon_mapped();
        // let photon_intensity = photon.intensity
        //     * (material.compute_once(self, photon.direction, &hit, 0)
        //         + material.compute_per_light(self, &ray.direction, &hit, &Vector::zero()));
        let photon_intensity = material.compute_photon(self, &hit, &incoming_photon.direction);
        // vary the position by a random amount (~0.001) to prevent issues with the kd tree
        let landed_photon = Photon::new(
            hit.position,
            incoming_photon.direction,
            photon_intensity,
            PhotonType::Diffuse,
        );
        // material.photon_landed(photon, self);
        // println!("Inserting photon at position {:?}", landed_photon.position.vector());
        self.photon_map.insert(landed_photon);
        // todo probabilities of diffuse etc
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
        for light in self.lights.iter() {
            light.shoot_photons(self, 1_000_000);
        }
    }

    fn raytrace(&self, ray: &Ray) -> RaytraceResult {
        let Some(hit) = self.trace(ray) else {
            return RaytraceResult {
                colour: Colour::black(),
                depth: 0.0,
            };
        };

        let neighbour_photons = self.photon_map.find_nearest(&hit.position, 100);
        let colour: Colour = neighbour_photons
            .iter()
            .map(|p| p.intensity)
            .fold(Colour::black(), |acc, intensity| acc + intensity)
            / neighbour_photons.len() as f32;

        RaytraceResult {
            colour,
            depth: hit.distance,
        }
    }

    fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }
}
