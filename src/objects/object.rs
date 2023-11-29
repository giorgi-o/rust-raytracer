use crate::core::{hit::HitVec, ray::Ray, transform::Transform};

pub trait Object: Send + Sync {
    fn intersect(&self, ray: &Ray) -> HitVec;
    fn apply_transform(&mut self, transform: &Transform);
}
