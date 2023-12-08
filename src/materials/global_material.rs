// The global material generates a reflection/refraction layer

use std::sync::Arc;

use crate::{
    core::{colour::Colour, hit::Hit, photon::Photon, ray::Ray, vector::Vector},
    environments::scene::Scene,
};

use super::material::{Material, PhotonBehaviour, PhotonMaterial, RefractionResult};

pub struct GlobalMaterial {
    reflect_weight: f32,
    refract_weight: f32,
    ior: f32, // index of refraction
}

impl GlobalMaterial {
    pub fn new(reflect_weight: f32, refract_weight: f32, ior: f32) -> Arc<Self> {
        Arc::new(Self {
            reflect_weight,
            refract_weight,
            ior,
        })
    }

    #[allow(non_snake_case)]
    fn refraction(&self, hit: &Hit, incoming: Vector) -> Option<RefractionResult> {
        if self.refract_weight == 0.0 {
            return None;
        }

        let mut N = hit.normal;
        let D = incoming;
        let I = -D;
        let mut cos_θ_i = N.dot(&I);

        let (η1, η2) = if hit.entering {
            N = -N;
            (1.0, self.ior)
        } else {
            cos_θ_i = -cos_θ_i;
            (self.ior, 1.0)
        };

        let cos_θ_t = (1.0 - (η1 / η2).powi(2) * (1.0 - cos_θ_i.powi(2))).sqrt();
        if cos_θ_t.is_nan() {
            return None; // total internal reflection
        }

        let T = I * (η1 / η2) - N * (cos_θ_t - (η1 / η2) * cos_θ_i);
        let T = -T.normalised();

        // use fresnel equations to determine reflectance
        let r_par = (η2 * cos_θ_i - η1 * cos_θ_t) / (η2 * cos_θ_i + η1 * cos_θ_t);
        let r_per = (η1 * cos_θ_i - η2 * cos_θ_t) / (η1 * cos_θ_i + η2 * cos_θ_t);
        let kr = (r_par.powi(2) + r_per.powi(2)) / 2.0; // reflectance coefficient

        // raytrace the refracted ray
        let refract_origin = hit.position.clone() + T * 0.0001;
        let refract_ray = Ray::new(refract_origin, T);

        Some(RefractionResult {
            ray: refract_ray,
            kr,
        })
    }
}

impl Material for GlobalMaterial {
    fn compute_once(&self, scene: &Scene, viewer: &Ray, hit: &Hit, depth: u8) -> Colour {
        if depth >= 5 {
            return Colour::black();
        }

        // reflection
        let mut reflection_colour = None;
        if self.reflect_weight > 0.0 {
            // spawn a reflection ray at the hit point
            let reflection_direction = hit.normal.reflection(&viewer.direction).normalised();
            let reflection_origin = hit.position.clone() + reflection_direction * 0.0001;
            let reflection_ray = Ray::new(reflection_origin, reflection_direction);

            reflection_colour =
                Some(scene.raytrace(&reflection_ray, depth + 1).colour * self.reflect_weight);
        }

        // refraction
        let mut refraction_colour = None;
        let mut kr = 0.0;
        if self.refract_weight > 0.0 {
            if let Some(refract_result) = self.refraction(hit, viewer.direction) {
                kr = refract_result.kr;

                let raytraced_colour = scene.raytrace(&refract_result.ray, depth + 1).colour;
                refraction_colour = Some(raytraced_colour * self.refract_weight);
            }
        }

        match (reflection_colour, refraction_colour) {
            (Some(reflection_colour), Some(refraction_colour)) => {
                (reflection_colour * (kr)) + (refraction_colour * (1.0 - kr))
            }
            (Some(reflection_colour), None) => reflection_colour,
            (None, Some(refraction_colour)) => refraction_colour,
            (None, None) => Colour::black(),
        }
    }

    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        self
    }
}

impl PhotonMaterial for GlobalMaterial {
    fn behaviour_weight(&self, behaviour: &PhotonBehaviour) -> f32 {
        match behaviour {
            PhotonBehaviour::Absorb => 0.0,
            PhotonBehaviour::Diffuse => 0.0,
            PhotonBehaviour::Specular => 0.0,
            PhotonBehaviour::ReflectOrRefract => 1.0,
        }
    }

    fn bounced_photon(&self, _photon: &Photon, _hit: &Hit) -> Option<Colour> {
        None // probability of photon diffusely bouncing off is 0
    }

    fn refract_chance(&self, kr: f32) -> f32 {
        let reflect = self.reflect_weight * kr;
        let refract = self.refract_weight * (1.0 - kr);
        refract / (reflect + refract)
    }
    fn refracted_direction(&self, hit: &Hit, incoming: Vector) -> Option<RefractionResult> {
        self.refraction(hit, incoming)
    }
}
