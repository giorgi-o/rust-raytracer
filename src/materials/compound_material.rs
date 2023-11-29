use std::sync::Arc;

use crate::core::{colour::Colour, environment::Environment, hit::Hit, ray::Ray, vector::Vector};

use super::{global_material::GlobalMaterial, material::Material, phong_material::Phong};

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
        let ambient = colour * 0.1;
        let diffuse = colour * 0.7;
        let specular = Colour::grey(0.5);
        let phong = Phong::new(ambient, diffuse, specular, 100.0);

        let global = GlobalMaterial::new(colour * reflectiveness, Colour::black(), 1.0);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }

    pub fn new_transparent(colour: Colour, transparency: f32, ior: f32) -> Arc<Self> {
        let opaqueness = 1.0 - transparency;
        let ambient = colour * 0.1 * opaqueness;
        let diffuse = colour * 0.7 * opaqueness;
        let specular = Colour::grey(0.5);
        let phong = Phong::new(ambient, diffuse, specular, 100.0);

        let global = GlobalMaterial::new(colour * transparency, colour * transparency, ior);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }
}

impl Material for CompoundMaterial {
    fn compute_once(&self, env: &dyn Environment, viewer: &Ray, hit: &Hit, recurse: u8) -> Colour {
        self.materials
            .iter()
            .fold(Colour::black(), |acc, material| {
                acc + material.compute_once(env, viewer, hit, recurse)
            })
    }

    fn compute_per_light(
        &self,
        env: &dyn Environment,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        self.materials
            .iter()
            .fold(Colour::black(), |acc, material| {
                acc + material.compute_per_light(env, viewer, hit, ldir)
            })
    }
}
