use std::sync::Arc;

use crate::{
    core::{
        hit::{Hit, HitVec},
        ray::Ray,
        transform::Transform,
        vector::Vector,
        vertex::Vertex,
    },
    hitvec,
    materials::material::Material,
};

use super::object::Object;

pub struct Plane {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    material: Arc<dyn Material>,
}

impl Plane {
    pub fn new_raw(a: f32, b: f32, c: f32, d: f32, material: Arc<dyn Material>) -> Self {
        Self {
            a,
            b,
            c,
            d,
            material,
        }
    }

    pub fn new_from_point(point: &Vertex, normal: &Vector, material: Arc<dyn Material>) -> Self {
        let d = -normal.dot(&point.vector());
        Self::new_raw(normal.x, normal.y, normal.z, d, material)
    }
}

impl Object for Plane {
    fn intersect(&self, ray: &Ray) -> HitVec {
        let material = self.material.as_ref();
        let u =
            self.a * ray.position.x + self.b * ray.position.y + self.c * ray.position.z + self.d;
        let v = self.a * ray.direction.x + self.b * ray.direction.y + self.c * ray.direction.z;

        if v == 0.0 {
            // ray is perfectly parallel to plane
            if u >= 0.0 {
                // ray is inside the plane
                let hit1 = Hit::infinity(self, true, f32::NEG_INFINITY, material);
                let hit2 = Hit::infinity(self, false, f32::INFINITY, material);

                return hitvec![hit1, hit2];
            } else {
                // ray is parallel alongside the plane
                return hitvec![];
            }
        }

        let t = u / -v;
        if v > 0.0 {
            let hit1 = Hit::infinity(self, true, f32::NEG_INFINITY, material);

            let mut hit2_normal = Vector::new(self.a, self.b, self.c);
            if hit2_normal.dot(&ray.direction) > 0.0 {
                hit2_normal.negate();
            }
            let hit2 = Hit::new(
                self,
                false,
                t,
                ray.position.clone() + ray.direction * t,
                hit2_normal,
                material,
                None,
            );

            hitvec![hit1, hit2]
        } else {
            // V < 0
            let mut hit1_normal = Vector::new(self.a, self.b, self.c);
            if hit1_normal.dot(&ray.direction) > 0.0 {
                hit1_normal.negate();
            }
            let hit1 = Hit::new(
                self,
                true,
                t,
                ray.position.clone() + ray.direction * t,
                hit1_normal,
                material,
                None,
            );

            let hit2 = Hit::infinity(self, false, f32::INFINITY, material);

            hitvec![hit1, hit2]
        }
    }

    fn apply_transform(&mut self, transform: &Transform) {
        let transform = transform.inverse().transposed();
        let mut v = Vertex::new(self.a, self.b, self.c, self.d);
        v.apply_transform(&transform);

        self.a = v.x;
        self.b = v.y;
        self.c = v.z;
        self.d = v.w;
    }
}
