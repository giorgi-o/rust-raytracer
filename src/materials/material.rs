use crate::{
    core::{
        colour::Colour, hit::Hit, photon::Photon, photon_tree::PhotonTree, ray::Ray,
        tex_coords::TexCoords, vector::Vector, vertex::Vertex,
    },
    environments::{photon_scene::PhotonScene, scene::Scene},
};

pub trait Material: Send + Sync {
    // called once per intersection
    fn compute_once(&self, scene: &Scene, viewer: &Ray, hit: &Hit, depth: u8) -> Colour;

    // called for each light that reaches a surface
    fn compute_per_light(&self, scene: &Scene, viewer: &Vector, hit: &Hit, ldir: &Vector)
        -> Colour;

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
}

pub trait PhotonMaterial: Material {
    // fn photon_tree(&self) -> &PhotonTree;

    fn behaviour_weight(&self, behaviour: &PhotonBehaviour) -> f32 {
        match behaviour {
            PhotonBehaviour::Absorb => 0.6,
            PhotonBehaviour::Diffuse => 0.3,
            PhotonBehaviour::Specular => 0.1,
        }
    }

    fn compute_photon(&self, scene: &PhotonScene, hit: &Hit, ldir: &Vector) -> Colour;

    // fn photon_landed(&self, photon: Photon, _scene: &PhotonScene) {
    //     // self.photon_tree().insert(photon);
    // }
    // fn photons_in_radius(&self, position: &Vertex, radius: f32) -> NeighbourPhotons {
    //     self.photon_tree().get_within_radius(position, radius)
    // }
}
