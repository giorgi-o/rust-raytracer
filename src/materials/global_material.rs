// The global material generates a reflection/refraction layer

use std::sync::Arc;

use crate::{core::{colour::Colour, hit::Hit, ray::Ray, vector::Vector}, environments::scene::Scene};

use super::material::Material;

pub struct GlobalMaterial {
    reflect_weight: Colour,
    refract_weight: Colour,
    ior: f32, // index of refraction
}

impl GlobalMaterial {
    pub fn new(reflect_weight: Colour, refract_weight: Colour, ior: f32) -> Arc<Self> {
        Arc::new(Self {
            reflect_weight,
            refract_weight,
            ior,
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
        if !self.reflect_weight.is_black() {
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
        #[allow(non_snake_case)]
        if !self.refract_weight.is_black() {
            let mut N = hit.normal;
            let D = viewer.direction;
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
            if !cos_θ_t.is_nan() {
                // not total internal reflection

                let T = I * (η1 / η2) - N * (cos_θ_t - (η1 / η2) * cos_θ_i);
                let T = -T.normalised();

                // use fresnel equations to determine reflectance
                let r_par = (η2 * cos_θ_i - η1 * cos_θ_t) / (η2 * cos_θ_i + η1 * cos_θ_t);
                let r_per = (η1 * cos_θ_i - η2 * cos_θ_t) / (η1 * cos_θ_i + η2 * cos_θ_t);
                kr = (r_par.powi(2) + r_per.powi(2)) / 2.0; // reflectance coefficient

                // raytrace the refracted ray
                let refract_origin = hit.position.clone() + T * 0.0001;
                let refract_ray = Ray::new(refract_origin, T);

                let raytraced_colour = scene.raytrace(&refract_ray, depth + 1).colour;
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

    fn compute_per_light(
        &self,
        _scene: &Scene,
        _viewer: &Vector,
        _hit: &Hit,
        _ldir: &Vector,
    ) -> Colour {
        Colour::black()
    }
}
