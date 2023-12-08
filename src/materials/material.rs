#![allow(unused_variables)]

use crate::{
    core::{
        colour::Colour, hit::Hit, photon::Photon, ray::Ray, tex_coords::TexCoords, vector::Vector,
    },
    environments::scene::Scene,
};

pub trait Material: Send + Sync {
    // called once per intersection
    fn compute_once(&self, scene: &Scene, viewer: &Ray, hit: &Hit, depth: u8) -> Colour;

    // called for each light that reaches a surface
    fn compute_per_light(
        &self,
        scene: &Scene,
        viewer: &Vector,
        hit: &Hit,
        ldir: &Vector,
    ) -> Colour {
        Colour::black()
    }

    // materials that support bump/normal maps should implement this
    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        None
    }

    // You will need additional material methods to support Photon-mapping.

    // assert this is a photon mapped material and return a reference to it
    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        panic!("Material does not support photon mapping");
    }
}

#[derive(Copy, Clone)]
pub enum PhotonBehaviour {
    Absorb,
    Diffuse,
    Specular,
    ReflectOrRefract,
}

pub struct RefractionResult {
    pub ray: Ray,
    pub kr: f32,
}

pub trait PhotonMaterial: Material {
    // fn photon_tree(&self) -> &PhotonTree;

    fn behaviour_weight(&self, behaviour: &PhotonBehaviour) -> f32 {
        match behaviour {
            PhotonBehaviour::Absorb => 0.6,
            PhotonBehaviour::Diffuse => 0.3,
            PhotonBehaviour::Specular => 0.1,
            PhotonBehaviour::ReflectOrRefract => 0.0,
        }
    }

    // these return None if the absorb, diffuse and specular weights are all 0
    // i.e. the object is transparent or mirror
    fn bounced_photon(&self, photon: &Photon, hit: &Hit) -> Option<Colour>;
    fn render_vueon(&self, hit: &Hit, photon: &Photon, viewer: Vector) -> Colour {
        Colour::black()
    }

    // these return None/0 if the object is neither reflective or transparent
    fn refract_chance(&self, kr: f32) -> f32 {
        0.0
    }
    fn refracted_direction(&self, hit: &Hit, viewer_incoming: Vector) -> Option<RefractionResult> {
        None
    }
}
