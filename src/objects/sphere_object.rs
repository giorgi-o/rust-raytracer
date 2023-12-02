use std::{f32::consts::PI, sync::Arc};

use crate::{
    core::{
        hit::{Hit, HitVec},
        ray::Ray,
        tex_coords::TexCoords,
        transform::Transform,
        vector::Vector,
        vertex::Vertex,
    },
    hitvec,
    materials::material::Material,
};

use super::object::Object;

pub struct Sphere {
    pub centre: Vertex,
    pub radius: f32,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(centre: Vertex, radius: f32, material: Arc<dyn Material>) -> Self {
        Self {
            centre,
            radius,
            material,
        }
    }
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray) -> HitVec {
        // offset ray by sphere position
        // equivalent to transforming ray into local sphere space
        let ro = ray.position.vector() - self.centre.vector();

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&ro);
        let c = ro.dot(&ro) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            // a negative discriminant means no intersection
            return hitvec![];
        }

        let ds = discriminant.sqrt();

        let t0 = (-b - ds) / 2.0;
        let t1 = (-b + ds) / 2.0;

        let create_hit = |distance, entering| {
            let position = ray.position.clone() + ray.direction * distance;
            let mut normal = (position.vector() - self.centre.vector()).normalised();
            if normal.dot(&ray.direction) > 0.0 {
                normal.negate();
            }

            let theta = (-normal.x.atan2(normal.z)) + PI; // longitude
            let phi = (-normal.y).acos(); // latitude
            let u = theta / (2.0 * PI);
            let v = (PI - phi) / PI;
            let tex_coords = TexCoords::new(u, v);

            if let Some(mut normal_map) = self.material.normal(&tex_coords) {
                // rotate the normal map
                // maths from https://computergraphics.stackexchange.com/a/5499
                let a = Vector::new(1.0, 0.0, 0.0);
                let tangent = a
                    .cross(&(position.clone() - self.centre.vector()).vector())
                    .normalised();
                normal_map = normal_map.to_tangent_space(&tangent, &normal);
                normal = normal_map.normalised();
            }

            Hit::new(
                self,
                entering,
                distance,
                position,
                normal,
                self.material.as_ref(),
                Some(tex_coords),
            )
        };

        hitvec![create_hit(t0, true), create_hit(t1, false)]
    }

    fn apply_transform(&mut self, transform: &Transform) {
        self.centre.apply_transform(transform);
    }
}
