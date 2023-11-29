use super::{ray::Ray, colour::Colour};

pub struct RaytraceResult {
    pub colour: Colour,
    pub depth: f32,
}

pub trait Environment: Send + Sync {
    // shoot a ray into the environment and get the colour and depth.
	// recurse indicates the level of recursion permitted.
    fn raytrace(&self, ray: &Ray, recurse: u8) -> RaytraceResult;

    // raytrace a shadow ray. returns true if intersection found between 0 and limit along ray.
    fn shadowtrace(&self, ray: &Ray, limit: f32) -> bool;
}