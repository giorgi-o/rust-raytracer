use crate::core::{ray::Ray, hit::Hit, vector::Vector, colour::Colour, environment::Environment, tex_coords::TexCoords};

pub trait Material: Send + Sync {
    // called once per intersection
    fn compute_once(&self, env: &dyn Environment, viewer: &Ray, hit: &Hit, recurse: u8) -> Colour;

    // called for each light that reaches a surface
    fn compute_per_light(&self, env: &dyn Environment, viewer: &Vector, hit: &Hit, ldir: &Vector) -> Colour;

    // materials that support bump/normal maps should implement this
    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        None
    }

    // You will need additional material methods to support Photon-mapping.
}
