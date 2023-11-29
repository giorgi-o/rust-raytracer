use crate::core::{colour::Colour, vector::Vector, vertex::Vertex};

pub trait Light: Send + Sync {
    // Get the direction towards the light at the point on the surface
    // return none if the surface is behind and not illuminated
    fn get_direction(&self, surface: &Vertex) -> Option<Vector>;

    // Get the intensity of the light in the direction of the surface
    fn get_intensity(&self, surface: &Vertex) -> Option<Colour>;

    // You will need additional light methods to support Photon-mapping.
}
