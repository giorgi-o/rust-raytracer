// Phong is a child class of Material and implement the simple Phong
// surface illumination model.

use std::sync::Arc;

use crate::core::{colour::Colour, environment::Environment, hit::Hit, ray::Ray, vector::Vector};

use super::material::Material;

pub struct Phong {
    pub ambient: Colour,
    pub diffuse: Colour,
    pub specular: Colour,
    pub shininess: f32,
}

impl Phong {
    pub fn new(ambient: Colour, diffuse: Colour, specular: Colour, shininess: f32) -> Arc<Self> {
        Arc::new(Self {
            ambient,
            diffuse,
            specular,
            shininess,
        })
    }
}

impl Material for Phong {
    fn compute_once(
        &self,
        _env: &dyn Environment,
        _viewer: &Ray,
        _hit: &Hit,
        _recurse: u8,
    ) -> Colour {
        self.ambient
    }

    fn compute_per_light(
        &self,
        _env: &dyn Environment,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        // Diffuse term
        let diffuse_strength = -hit.normal.dot(ldir);
        let mut result = self.diffuse * diffuse_strength;

        // Specular term
        let reflection = hit.normal.reflection(ldir).normalised();
        let specular_strength = viewer.dot(&reflection).max(0.0).powf(self.shininess);
        result += self.specular * specular_strength;

        result
    }
}
