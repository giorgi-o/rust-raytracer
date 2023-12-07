use std::io::Write;

use kd_tree::ItemAndDistance;
use rand::{distributions::Uniform, seq::SliceRandom, Rng};

use crate::{
    core::{
        colour::Colour,
        hit::Hit,
        photon::{InFlightPhoton, Photon, PhotonType},
        photon_tree::PhotonTree,
        ray::Ray,
        vector::Vector,
        vertex::Vertex,
    },
    lights::light::{Light, PhotonLight},
    materials::material::{Material, PhotonBehaviour, PhotonMaterial},
    objects::object::Object,
};

use super::environment::{Environment, RaytraceResult};

const PHOTONS_PER_LIGHT: usize = 5_000_000;
const CAUSTIC_PHOTONS_PER_LIGHT: usize = 1_000_000;

pub struct PhotonScene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn PhotonLight>>,
    regular_photon_map: Option<PhotonTree>,
    caustic_photon_map: Option<PhotonTree>,
}

impl PhotonScene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            regular_photon_map: None,
            caustic_photon_map: None,
        }
    }

    fn build_regular_photon_map(&mut self) {
        self.build_photon_map(|this, light| light.shoot_photons_mt(this, PHOTONS_PER_LIGHT as u32));
    }

    fn shoot_regular_photons(&self)

    fn build_photon_map(
        &mut self,
        shoot_photons: impl Fn(&Self, &Box<dyn PhotonLight>) -> Vec<Vec<Photon>>,
    ) {
        let mut photons = Vec::new();

        let lights = std::mem::take(&mut self.lights);
        for light in lights.iter() {
            // let light_photons = light.shoot_photons_mt(self, PHOTONS_PER_LIGHT as u32);
            let light_photons = shoot_photons(self, light);

            println!("Light emitted {} photons", light_photons.len());
            photons.push(light_photons);
        }
        self.lights = lights;

        let photons = self.flatten_photons(photons);

        println!("\nBuilding photon KD tree...");
        self.regular_photon_map = Some(PhotonTree::build(photons));
    }

    fn flatten_photons(&self, mut photons: Vec<Vec<Vec<Photon>>>) -> Vec<Photon> {
        let mut flat_photons = Vec::new();
        let mut stdout = std::io::stdout().lock();
        let mut i = 0;
        let max_i = photons.len();
        while let Some(mut vec_vec) = photons.pop() {
            i += 1;
            let mut j = 0;
            let max_j = vec_vec.len();

            while let Some(vec) = vec_vec.pop() {
                j += 1;
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
        flat_photons
    }

    pub fn photontrace(&self, photon: InFlightPhoton) -> Vec<Photon> {
        let ray = photon.ray();
        let Some(hit) = self.trace(&ray) else {
            return Vec::new();
        };

        let material = hit.material.photon_mapped();

        // pick absorb, diffuse or specular based on weights
        let mut rng = rand::thread_rng();
        let choice = [
            PhotonBehaviour::Absorb,
            PhotonBehaviour::Diffuse,
            PhotonBehaviour::Specular,
            PhotonBehaviour::ReflectOrRefract,
        ]
        .choose_weighted(&mut rng, |item| material.behaviour_weight(item))
        .unwrap();

        let (absorbed_photon, shadow_photons) = self.absorb_photon(photon, &hit);

        let bounced_photons = match choice {
            PhotonBehaviour::Absorb => Vec::new(),
            PhotonBehaviour::Diffuse => self.diffuse_photon(&absorbed_photon, &hit),
            PhotonBehaviour::Specular => self.specular_photon(&absorbed_photon, &hit),
            PhotonBehaviour::ReflectOrRefract => {
                self.reflect_or_refract_photon(&absorbed_photon, &ray, &hit, material)
            }
        };

        let mut photons = bounced_photons;
        photons.push(absorbed_photon);
        photons.extend(shadow_photons);

        photons
    }

    fn absorb_photon(&self, photon: InFlightPhoton, hit: &Hit) -> (Photon, Vec<Photon>) {
        // store photon in kd tree
        let absorbed_photon = Photon::new(
            hit.position.clone(),
            photon.direction,
            photon.intensity,
            PhotonType::Colour,
        );

        let shadow_photons = self.shadowphotontrace(&absorbed_photon);

        (absorbed_photon, shadow_photons)
    }

    fn shadowphotontrace(&self, absorbed_photon: &Photon) -> Vec<Photon> {
        let ray = Ray::new(
            absorbed_photon.position.clone() + absorbed_photon.incident * 0.0001,
            absorbed_photon.incident,
        );

        let mut shadow_photons = Vec::new();

        for object in self.objects.iter() {
            let hits = object.intersect(&ray);
            for hit in hits {
                if !hit.entering || hit.distance < 0.0 {
                    continue;
                }

                let shadow_photon = Photon::new(
                    hit.position.clone(),
                    ray.direction,
                    Colour::black(),
                    PhotonType::Shadow,
                );
                shadow_photons.push(shadow_photon);
            }
        }

        shadow_photons
    }

    fn diffuse_photon(&self, photon: &Photon, hit: &Hit) -> Vec<Photon> {
        let mut direction = Vector::random();

        // flip direction if it's facing away from the normal
        if direction.dot(&hit.normal) < 0.0 {
            direction.negate();
        }

        let intensity = hit
            .material
            .photon_mapped()
            .bounced_photon(photon, hit)
            .unwrap();
        let photon = InFlightPhoton::new(
            hit.position.clone(),
            direction.normalised(),
            intensity,
            PhotonType::Colour,
        );

        self.photontrace(photon)
    }

    fn specular_photon(&self, photon: &Photon, hit: &Hit) -> Vec<Photon> {
        let reflection = hit.normal.reflection(&photon.incident).normalised();

        let intensity = hit
            .material
            .photon_mapped()
            .bounced_photon(photon, hit)
            .unwrap();
        let photon = InFlightPhoton::new(
            hit.position.clone(),
            reflection,
            intensity,
            PhotonType::Colour,
        );

        self.photontrace(photon)
    }

    fn reflect_or_refract_photon(
        &self,
        photon: &Photon,
        ray: &Ray,
        hit: &Hit,
        material: &dyn PhotonMaterial,
    ) -> Vec<Photon> {
        let reflect_direction = hit.normal.reflection(&photon.incident).normalised();
        let reflected_photon = || {
            InFlightPhoton::new(
                hit.position.clone() + reflect_direction * 0.0001,
                reflect_direction,
                photon.intensity,
                PhotonType::Caustic,
            )
        };

        let Some(refract_result) = material.refracted_direction(&hit, ray.direction) else {
            return self.photontrace(reflected_photon());
        };

        // pick reflection or refraction
        let refract_chance = material.refract_chance(refract_result.kr);
        let mut rng = rand::thread_rng();
        let should_refract = rng.gen_bool(refract_chance as f64);

        if should_refract {
            self.photontrace(InFlightPhoton::new(
                refract_result.ray.position,
                refract_result.ray.direction,
                photon.intensity,
                PhotonType::Caustic,
            ))
        } else {
            self.photontrace(reflected_photon())
        }
    }

    fn vueontrace(&self, vueon: InFlightPhoton) -> RaytraceResult {
        let ray = vueon.ray();
        let Some(hit) = self.trace(&ray) else {
            return RaytraceResult::none();
        };

        let material = hit.material.photon_mapped();

        // calculate regular surface colour (no reflection/refraction)
        let surface_weight = material.behaviour_weight(&PhotonBehaviour::Absorb)
            + material.behaviour_weight(&PhotonBehaviour::Diffuse)
            + material.behaviour_weight(&PhotonBehaviour::Specular);
        let mut surface_colour = Colour::black();
        if surface_weight > 0.0 {
            if let Some(photon) = self.average_photon_at(&hit) {
                surface_colour = material.render_vueon(&hit, &photon, -vueon.direction);
            }
        }

        // calculate reflection colour
        let reflect_weight = material.behaviour_weight(&PhotonBehaviour::ReflectOrRefract);
        let reflect_vueon = InFlightPhoton::new(
            hit.position.clone() + hit.normal * 0.0001,
            hit.normal.reflection(&vueon.direction).normalised(),
            vueon.intensity,
            PhotonType::Colour,
        );
        let mut reflect_colour = Colour::black();
        if reflect_weight > 0.0 {
            reflect_colour = self.vueontrace(reflect_vueon).colour * reflect_weight;
        }

        // calculate refraction colour
        let refract_weight = material.behaviour_weight(&PhotonBehaviour::ReflectOrRefract);
        let mut refract_colour = Colour::black();
        if let Some(refract_result) = material.refracted_direction(&hit, ray.direction) {
            let refract_vueon = InFlightPhoton::new(
                refract_result.ray.position + refract_result.ray.direction * 0.0001,
                refract_result.ray.direction,
                vueon.intensity,
                PhotonType::Colour,
            );
            refract_colour = self.vueontrace(refract_vueon).colour * refract_weight
        }

        let mut colour = surface_colour + reflect_colour + refract_colour;
        colour = colour / (surface_weight + reflect_weight + refract_weight);

        RaytraceResult {
            colour,
            depth: hit.distance,
        }
    }

    fn average_photon_at(&self, hit: &Hit) -> Option<Photon> {
        let mut neighbour_photons = self
            .regular_photon_map
            .as_ref()
            .expect("Photon map not built")
            .get_within_distance(&hit.position, 0.1);
        let photons_in_radius = neighbour_photons.len();
        if photons_in_radius == 0 {
            return None;
        }

        let neighbour_photons_len = neighbour_photons.len() as f32;
        let material = hit.material.photon_mapped();

        let mut average_ldir = Vector::new(0.0, 0.0, 0.0);
        let mut average_intensity = Colour::black();

        for ItemAndDistance {
            item: photon,
            squared_distance,
        } in neighbour_photons
        {
            average_ldir += photon.incident.normalised();
            average_intensity += photon.intensity;
        }

        average_ldir.normalise();
        average_intensity = average_intensity / neighbour_photons_len;

        let photon = Photon::new(
            hit.position.clone(),
            average_ldir,
            average_intensity,
            PhotonType::Colour,
        );

        Some(photon)
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
        self.build_regular_photon_map();
    }

    fn raytrace(&self, ray: &Ray) -> RaytraceResult {
        let vueon = InFlightPhoton::new(
            ray.position.clone(),
            ray.direction,
            Colour::white(),
            PhotonType::Vueon,
        );

        self.vueontrace(vueon)
    }

    fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }
}
