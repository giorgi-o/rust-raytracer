use crate::{core::{colour::Colour, vector::Vector, vertex::Vertex}, environments::photon_scene::PhotonScene};

pub trait Light: Send + Sync {
    // Get the direction towards the light at the point on the surface
    // return none if the surface is behind and not illuminated
    fn get_direction(&self, surface: &Vertex) -> Option<Vector>;

    // Get the intensity of the light in the direction of the surface
    fn get_intensity(&self, surface: &Vertex) -> Option<Colour>;

    // You will need additional light methods to support Photon-mapping.

    fn photon_light(self: Box<Self>) -> Box<dyn PhotonLight> {
        panic!("Light does not support photon mapping");
    }
}

pub trait PhotonLight: Light {
    fn shoot_photons(&self, scene: &PhotonScene, num_photons: u32);
}
