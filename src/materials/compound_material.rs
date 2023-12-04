use std::sync::Arc;

use crate::{
    core::{colour::Colour, hit::Hit, ray::Ray, vector::Vector},
    environments::scene::Scene,
};

use super::{
    global_material::GlobalMaterial,
    material::Material,
    phong_material::{Monochrome, Phong},
    texture::Texture,
};

pub struct CompoundMaterial {
    materials: Vec<Arc<dyn Material>>,
}

impl CompoundMaterial {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
        }
    }

    pub fn add_material(&mut self, material: Arc<impl Material + 'static>) {
        // self.materials.push(material);
        self.materials.push(material);
    }

    pub fn new_simple(colour: Colour, reflectiveness: f32) -> Arc<Self> {
        let phong = Monochrome::new(colour, 0.1, 100.0);

        let global = GlobalMaterial::new(colour * reflectiveness, Colour::black(), 1.0);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }

    pub fn new_translucent(colour: Colour, transparency: f32, ior: f32) -> Arc<Self> {
        let opaqueness = 1.0 - transparency;
        let phong = Monochrome::new(colour * opaqueness, 0.1, 100.0);

        let global = GlobalMaterial::new(colour * transparency, colour * transparency, ior);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }
}

impl Material for CompoundMaterial {
    fn compute_once(&self, scene: &Scene, viewer: &Ray, hit: &Hit, depth: u8) -> Colour {
        self.materials
            .iter()
            .fold(Colour::black(), |acc, material| {
                acc + material.compute_once(scene, viewer, hit, depth)
            })
    }

    fn compute_per_light(
        &self,
        scene: &Scene,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        self.materials
            .iter()
            .fold(Colour::black(), |acc, material| {
                acc + material.compute_per_light(scene, viewer, hit, ldir)
            })
    }
}
