use std::sync::Arc;

use crate::{
    core::{
        colour::Colour, hit::Hit, photon::Photon, ray::Ray, tex_coords::TexCoords, vector::Vector,
    },
    environments::scene::Scene,
};

use super::material::{Material, PhotonMaterial};

pub trait Phong: Send + Sync {
    fn colour_at_hit(&self, hit: &Hit) -> Colour;
    fn ambient_strength(&self) -> f32;
    fn shininess(&self) -> f32;
    fn normal(&self, _tex_coords: &TexCoords) -> Option<Vector> {
        None
    }
    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        panic!("Material does not support photon mapping");
    }

    fn ambient(&self, hit: &Hit) -> Colour {
        self.colour_at_hit(hit) * self.ambient_strength()
    }
    fn diffuse(&self, hit: &Hit, ldir: &Vector) -> Colour {
        let diffuse_strength = -hit.normal.dot(ldir);
        self.colour_at_hit(hit) * diffuse_strength
    }
    fn specular(&self, hit: &Hit, ldir: &Vector, viewer: &Vector) -> Colour {
        let reflection = hit.normal.reflection(ldir).normalised();
        let specular_strength = viewer.dot(&reflection).max(0.0).powf(self.shininess());
        Colour::white() * specular_strength
    }
}

// impl material for PhongT
impl<T: Phong> Material for T {
    fn compute_once(&self, _scene: &Scene, _viewer: &Ray, hit: &Hit, _depth: u8) -> Colour {
        self.ambient(hit)
    }

    fn compute_per_light(
        &self,
        _scene: &Scene,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        self.diffuse(hit, ldir) + self.specular(hit, ldir, viewer)
    }

    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        self.normal(tex_coords)
    }

    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        self.photon_mapped()
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

    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        self
    }
}

impl<T: Phong> PhotonMaterial for T {
    fn bounced_photon(&self, photon: &Photon, hit: &Hit) -> Option<Colour> {
        Some(self.diffuse(hit, &photon.incident) * photon.intensity)
    }

    fn render_vueon(&self, hit: &Hit, photon: &Photon, viewer: Vector) -> Colour {
        let colour =
            self.diffuse(hit, &photon.incident) + self.specular(hit, &photon.incident, &viewer);
        colour * photon.intensity
    }
}
