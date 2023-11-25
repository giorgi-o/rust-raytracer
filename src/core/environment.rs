use super::{ray::Ray, colour::Colour};

pub trait Environment {
    // shoot a ray into the environment and get the colour and depth.
	// recurse indicates the level of recursion permitted.
    fn raytrace(ray: Ray, recurse: u32) -> (Colour, f32);

    // raytrace a shadow ray. returns true if intersection found between 0 and limit along ray.
    fn shadowtrace(ray: Ray, limit: f32) -> bool;
}