// This Material class maps the x,y,z components of the normal to the r,g,b components
// of the returned colour. A useful debug tool.

use crate::{
    core::{colour::Colour, hit::Hit, ray::Ray, vector::Vector},
    environments::scene::Scene,
};

use super::material::Material;

pub struct FalseColour {}

impl FalseColour {
    pub fn new() -> Self {
        Self {}
    }
}

impl Material for FalseColour {
    fn compute_once(&self, _scene: &Scene, _viewer: &Ray, hit: &Hit, _depth: u8) -> Colour {
        Colour::new(
            (hit.normal.x + 1.0) * 0.5,
            (hit.normal.y + 1.0) * 0.5,
            (hit.normal.z + 1.0) * 0.5,
        )
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
