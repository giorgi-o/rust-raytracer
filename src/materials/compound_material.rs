use std::sync::Arc;

use crate::{
    core::{
        colour::Colour, hit::Hit, photon::Photon, ray::Ray, tex_coords::TexCoords, vector::Vector,
    },
    environments::scene::Scene,
};

use super::{
    global_material::GlobalMaterial,
    material::{Material, PhotonBehaviour, PhotonMaterial, RefractionResult},
    phong_material::Monochrome,
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

    fn photon_materials(&self) -> impl Iterator<Item = &dyn PhotonMaterial> {
        self.materials
            .iter()
            .map(|material| material.photon_mapped())
    }

    pub fn new_simple(colour: Colour, reflectiveness: f32, shininess: f32) -> Arc<Self> {
        let phong = Monochrome::new(colour, 0.1, shininess);

        let global = GlobalMaterial::new(reflectiveness, 0.0, 1.0);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }

    pub fn new_translucent(
        colour: Colour,
        transparency: f32,
        ior: f32,
        shininess: f32,
    ) -> Arc<Self> {
        let opaqueness = 1.0 - transparency;
        let phong = Monochrome::new(colour * opaqueness, 0.1, shininess);

        let global = GlobalMaterial::new(transparency, transparency, ior);

        let mut compound = Self::new();
        compound.add_material(phong);
        compound.add_material(global);
        Arc::new(compound)
    }

    pub fn new_textured(texture: String, scale: f32, transparency: f32) -> Arc<Self> {
        let texture = Texture::import(texture.to_string(), scale, 0.1, 1000000.0);
        // let texture = Arc::new(FalseColour::new());
        let global = GlobalMaterial::new(transparency, transparency, 1.0);

        let mut compound = Self::new();
        compound.add_material(texture);
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
            / self.materials.len() as f32
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
            / self.materials.len() as f32
    }

    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        // return the first non-None result
        // this is fine for now because only one of our materials has
        // tetures. not ideal though.

        for material in self.photon_materials() {
            if let Some(result) = material.normal(tex_coords) {
                return Some(result);
            }
        }

        None
    }

    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        self
    }
}

impl PhotonMaterial for CompoundMaterial {
    fn behaviour_weight(&self, behaviour: &PhotonBehaviour) -> f32 {
        self.photon_materials().fold(0.0, |acc, material| {
            acc + material.behaviour_weight(behaviour)
        }) / self.materials.len() as f32
    }

    fn bounced_photon(&self, photon: &Photon, hit: &Hit) -> Option<Colour> {
        self.photon_materials()
            .fold(None, |acc, material| match acc {
                Some(colour) => match material.bounced_photon(photon, hit) {
                    Some(new_colour) => Some(colour + new_colour),
                    None => Some(colour),
                },
                None => material.bounced_photon(photon, hit),
            })
    }

    fn render_vueon(&self, hit: &Hit, photon: &Photon, viewer: Vector) -> Colour {
        self.photon_materials()
            .fold(Colour::black(), |acc, material| {
                acc + material.render_vueon(hit, photon, viewer)
            })
    }

    fn refract_chance(&self, kr: f32) -> f32 {
        self.photon_materials()
            .fold(0.0, |acc, material| acc + material.refract_chance(kr))
            / self.materials.len() as f32
    }

    fn refracted_direction(&self, hit: &Hit, viewer: Vector) -> Option<RefractionResult> {
        // return the first non-None result
        // this is fine for now because only one of our materials has
        // refraction. ideally we would randomly pick using the refract weight
        // of each material.

        for material in self.photon_materials() {
            if let Some(result) = material.refracted_direction(hit, viewer) {
                return Some(result);
            }
        }

        None
    }
}
