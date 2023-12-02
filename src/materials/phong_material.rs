use std::sync::Arc;

use crate::core::{colour::Colour, environment::Environment, hit::Hit, ray::Ray, vector::Vector, tex_coords::TexCoords};

use super::material::Material;

pub trait Phong: Send + Sync {
    fn colour_at_hit(&self, hit: &Hit) -> Colour;
    fn ambient_strength(&self) -> f32;
    fn shininess(&self) -> f32;
    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        None
    }
}

// impl material for PhongT
impl<T: Phong> Material for T {
    fn compute_once(
        &self,
        _env: &dyn Environment,
        _viewer: &Ray,
        hit: &Hit,
        _recurse: u8,
    ) -> Colour {
        self.colour_at_hit(hit) * self.ambient_strength()
    }

    fn compute_per_light(
        &self,
        env: &dyn Environment,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        // Diffuse term
        let diffuse_strength = -hit.normal.dot(ldir);
        let mut result = self.colour_at_hit(hit) * diffuse_strength;

        // Specular term
        let reflection = hit.normal.reflection(ldir).normalised();
        let specular_strength = viewer.dot(&reflection).max(0.0).powf(self.shininess());
        result += Colour::white() * specular_strength;

        result
    }

    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        self.normal(tex_coords)
    }
}

pub struct Monochrome {
    colour: Colour,
    ambient_strength: f32,
    shininess: f32,
}

impl Monochrome {
    pub fn new(colour: Colour, ambient_strength: f32, shininess: f32) -> Arc<Self> {
        Arc::new(Self {
            colour,
            ambient_strength,
            shininess,
        })
    }
}

impl Phong for Monochrome {
    fn colour_at_hit(&self, _hit: &Hit) -> Colour {
        self.colour
    }

    fn ambient_strength(&self) -> f32 {
        self.ambient_strength
    }

    fn shininess(&self) -> f32 {
        self.shininess
    }
}

// impl Material for Monochrome {
//     fn compute_once(
//         &self,
//         _env: &dyn Environment,
//         _viewer: &Ray,
//         _hit: &Hit,
//         _recurse: u8,
//     ) -> Colour {
//         self.ambient
//     }

//     fn compute_per_light(
//         &self,
//         _env: &dyn Environment,
//         viewer: &Vector,
//         hit: &Hit,
//         ldir: &Vector,
//     ) -> Colour {
//         // Diffuse term
//         let diffuse_strength = -hit.normal.dot(ldir);
//         let mut result = self.diffuse * diffuse_strength;

//         // Specular term
//         let reflection = hit.normal.reflection(ldir).normalised();
//         let specular_strength = viewer.dot(&reflection).max(0.0).powf(self.shininess);
//         result += self.specular * specular_strength;

//         result
//     }
// }
