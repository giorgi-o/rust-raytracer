use crate::core::hit::{Hit, HitVec};
use crate::{lights::light::Light, objects::object::Object};

use crate::core::{colour::Colour, ray::Ray};

use super::environment::{Environment, RaytraceResult};

pub struct Scene {
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Box<dyn Light>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    fn select_first_hit<'s>(&self, hits: HitVec<'s>) -> Option<Hit<'s>> {
        let mut min_hit: Option<Hit> = None;
        let mut min_distance = std::f32::MAX;

        for hit in hits {
            if hit.distance < 0.0 {
                continue;
            }
            if !hit.entering {
                continue;
            }

            if hit.distance < min_distance {
                min_distance = hit.distance;
                min_hit = Some(hit);
            }
        }

        min_hit
    }

    fn trace(&self, ray: &Ray) -> Option<Hit> {
        let mut min_hit: Option<Hit> = None;
        let mut min_distance = std::f32::MAX;

        for object in self.objects.iter() {
            let hits = object.intersect(ray);

            let hit = self.select_first_hit(hits);
            let Some(hit) = hit else {
                continue;
            };

            if hit.distance < min_distance {
                min_distance = hit.distance;
                min_hit = Some(hit);
            }
        }

        min_hit
    }

    // raytrace a shadow ray.
    // returns true if intersection found between 0 and limit along ray.
    fn shadowtrace(&self, ray: &Ray, limit: f32) -> bool {
        for object in self.objects.iter() {
            let hits = object.intersect(ray);
            let hit = self.select_first_hit(hits);
            let Some(hit) = hit else {
                continue;
            };

            if hit.distance > 0.0000001 && hit.distance < limit {
                return true;
            }
        }

        false
    }

    // shoot a ray into the environment and get the colour and depth.
    // depth indicates the current recursion level.
    pub fn raytrace(&self, ray: &Ray, depth: u8) -> RaytraceResult {
        // first step, find the closest primitive
        let Some(hit) = self.trace(ray) else {
            return RaytraceResult {
                colour: Colour::black(),
                depth: 0.0,
            };
        };

        // next, compute the colour we should see
        let mut colour = hit.material.compute_once(self, ray, &hit, depth);

        // then, compute the light contribution for every light in the scene
        for light in self.lights.iter() {
            let viewer = -hit.position.clone().vector().normalised();

            let mut lit = light.get_direction(&hit.position);
            if lit.as_ref().is_some_and(|ldir| ldir.dot(&hit.normal) > 0.0) {
                lit = None; // light is facing the wrong way
            }

            // shadow check
            if let Some(ldir) = lit {
                let mut shadow_ray = Ray::new(hit.position.clone(), -ldir);

                // add a small offset to the shadow ray origin to avoid self intersection
                shadow_ray.position += shadow_ray.direction * 0.0001;

                if self.shadowtrace(&shadow_ray, ldir.length()) {
                    lit = None;
                }
            }

            if let Some(ldir) = lit {
                let intensity = light
                    .get_intensity(&hit.position)
                    .expect("light.get_intensity() is None despite get_direction() being Some");
                colour += hit.material.compute_per_light(self, &viewer, &hit, &ldir) * intensity;
            }
        }

        RaytraceResult {
            colour,
            depth: hit.distance,
        }
    }
}

impl Environment for Scene {
    fn add_object(&mut self, object: Box<dyn Object + 'static>) {
        self.objects.push(object);
    }

    fn add_light(&mut self, light: Box<dyn Light + 'static>) {
        self.lights.push(light);
    }

    fn pre_render(&mut self) {}

    fn raytrace(&self, ray: &Ray) -> RaytraceResult {
        Scene::raytrace(self, ray, 0)
    }

    fn objects(&self) -> &[Box<dyn Object>] {
        &self.objects
    }
}
