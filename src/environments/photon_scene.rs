use crate::{
    core::{
        colour::Colour,
        hit::{Hit, HitVec},
        photon::Photon,
        photon_tree::PhotonTree,
        ray::Ray,
        vector::Vector,
    },
    lights::light::PhotonLight,
    objects::object::Object,
};

use super::environment::{Environment, RaytraceResult};

pub struct PhotonScene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn PhotonLight>>,
    photon_map: PhotonTree,
}

impl PhotonScene {
    pub fn photontrace(&self, photon: Photon) {
        let ray = photon.ray();
        let Some(hit) = self.trace(&ray) else {
            return;
        };

        let material = hit.material.photon_mapped();
        let photon_intensity = photon.intensity
            * (material.compute_once(self, photon.direction, &hit, 0)
                + material.compute_per_light(self, &ray.direction, &hit, &Vector::zero()));
        // material.photon_landed(photon, self);
        self.photon_map.insert(photon);
        // todo probabilities of diffuse etc
    }
}

impl Environment for PhotonScene {
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

        let photons = self.photon_map.find_nearest(&hit.position, 100);
        let mut colour = Colour::black();
        for photon in photons.iter() {}

        todo!()
    }

    fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }
}
