use std::sync::Arc;

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

pub struct Plane {
    centre: Vertex,
    up: Vector,
    normal: Vector,
    d: f32,
    material: Arc<dyn Material>,
}

impl Plane {
    pub fn new_raw(
        point: &Vertex,
        up: Vector,
        normal: Vector,
        material: Arc<dyn Material>,
    ) -> Self {
        Self {
            centre: point.clone(),
            up,
            normal,
            d: -normal.dot(&point.vector()),
            material,
        }
    }

    pub fn new(
        point: &Vertex,
        up: Vector,
        normal: Vector,
        material: Arc<dyn Material>,
    ) -> Box<Self> {
        Box::new(Self::new_raw(point, up, normal, material))
    }
}

impl Object for Plane {
    #[allow(non_snake_case)]
    fn intersect(&self, ray: &Ray) -> HitVec {
        let material = self.material.as_ref();
        let (a, b, c) = (self.normal.x, self.normal.y, self.normal.z);
        // let U =
        //     a * ray.position.x + b * ray.position.y + c * ray.position.z + self.d;
        // let V = a * ray.direction.x + b * ray.direction.y + c * ray.direction.z;
        let U = self.normal.dot(&ray.position.vector()) + self.d;
        let V = self.normal.dot(&ray.direction);

        if V == 0.0 {
            // ray is perfectly parallel to plane
            if U >= 0.0 {
                // ray is inside the plane
                let hit1 = Hit::infinity(self, true, f32::NEG_INFINITY, material);
                let hit2 = Hit::infinity(self, false, f32::INFINITY, material);

                return hitvec![hit1, hit2];
            } else {
                // ray is parallel alongside the plane
                return hitvec![];
            }
        }

        let t = U / -V;
        if V > 0.0 {
            let hit1 = Hit::infinity(self, true, f32::NEG_INFINITY, material);

            let mut normal = self.normal.negated();
            let hit2 = Hit::new(
                self,
                false,
                t,
                ray.position.clone() + ray.direction * t,
                normal,
                material,
                None,
            );

            hitvec![hit1, hit2]
        } else {
            // V < 0
            let mut normal = self.normal;
            let position = ray.position.clone() + ray.direction * t;

            let from_center = position.vector() - self.centre.vector();
            let v_unit_vector = self.up.normalised();
            let u_unit_vector = v_unit_vector.cross(&self.normal).normalised();
            let u = from_center.dot(&u_unit_vector);
            let v = from_center.dot(&v_unit_vector);
            // let u = (position.x.rem_euclid(1.0).powi(2) + position.y.powi(2)).sqrt();
            // let v = (position.z.rem_euclid(1.0).powi(2) + position.y.powi(2)).sqrt();
            let tex_coords = TexCoords::new(u, v);

            if let Some(normal_map) = self.material.normal(&tex_coords) {
                let right = self.normal.cross(&self.up).normalised();
                normal = normal_map.to_tangent_space(&right, &normal);
            }

            let hit1 = Hit::new(self, true, t, position, normal, material, Some(tex_coords));

            let hit2 = Hit::infinity(self, false, f32::INFINITY, material);

            hitvec![hit1, hit2]
        }
    }

    fn apply_transform(&mut self, transform: &Transform) {
        self.centre.apply_transform(transform);
        self.up.apply_transform(transform);
        self.normal.apply_transform(transform);

        self.normal.normalise();
        self.up.normalise();

        self.d = -self.normal.dot(&self.centre.vector());
    }
}
