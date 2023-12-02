use std::sync::{Arc, OnceLock};

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

use super::{object::Object, plane_object::Plane};

pub struct CuboidPlanes {
    left: Plane,
    right: Plane,
    up: Plane,
    down: Plane,
    front: Plane,
    back: Plane,
}

impl CuboidPlanes {
    fn iter(&self) -> impl Iterator<Item = &Plane> {
        [
            &self.left,
            &self.right,
            &self.up,
            &self.down,
            &self.front,
            &self.back,
        ]
        .into_iter()
    }
}

pub struct Cuboid {
    pub corner: Vertex, // bottom left corner
    pub size: Vector,
    material: Arc<dyn Material>,

    planes: OnceLock<CuboidPlanes>,
}

impl Cuboid {
    pub fn new(corner: Vertex, size: Vector, material: Arc<dyn Material>) -> Self {
        Self {
            corner,
            size,
            material,
            planes: OnceLock::new(),
        }
    }

    fn get_planes(&self) -> &CuboidPlanes {
        self.planes.get_or_init(|| {
            let corner = self.corner.clone();
            let Vector {
                x: width,
                y: height,
                z: depth,
            } = self.size;

            let fdl = corner.clone(); // front down left
            let ful = corner.clone() + Vector::new(0.0, height, 0.0);
            let bdl = corner.clone() + Vector::new(0.0, 0.0, depth);
            let bdr = corner.clone() + Vector::new(width, 0.0, depth);

            // vectors
            let up = Vector::new(0.0, 1.0, 0.0);
            let down = Vector::new(0.0, -1.0, 0.0);
            let left = Vector::new(-1.0, 0.0, 0.0);
            let right = Vector::new(1.0, 0.0, 0.0);
            let forwards = Vector::new(0.0, 0.0, 1.0);
            let backwards = Vector::new(0.0, 0.0, -1.0);

            let m = &self.material;
            //  pub fn new_from_point(point: &Vertex, up: Vector, normal: Vector, material: Arc<dyn Material>)
            CuboidPlanes {
                right: Plane::new_from_point(&bdr, up, right, m.clone()),
                left: Plane::new_from_point(&fdl, up, left, m.clone()),
                up: Plane::new_from_point(&ful, forwards, up, m.clone()),
                down: Plane::new_from_point(&fdl, forwards, down, m.clone()),
                front: Plane::new_from_point(&fdl, up, backwards, m.clone()),
                back: Plane::new_from_point(&bdl, down, forwards, m.clone()),
            }
        })
    }
}

impl Object for Cuboid {
    fn intersect(&self, ray: &Ray) -> HitVec {
        let planes = self.get_planes();
        let mut first_hit = None::<Hit>;
        let mut back_hit = None::<Hit>;

        for plane in planes.iter() {
            let hits = plane.intersect(ray);

            for hit in hits {
                // check if hit position is inside the cube
                let hit_position = &hit.position;
                let corner = &self.corner;
                let size = self.size;
                let inside = hit_position.x >= corner.x - 0.0001
                    && hit_position.x <= corner.x + size.x + 0.0001
                    && hit_position.y >= corner.y - 0.0001
                    && hit_position.y <= corner.y + size.y + 0.0001
                    && hit_position.z >= corner.z - 0.0001
                    && hit_position.z <= corner.z + size.z + 0.0001;
                if !inside {
                    continue;
                }

                if hit.distance < 0.0 {
                    continue; // behind the camera
                }
                if hit.distance.is_infinite() {
                    continue; // parallel to the plane
                }

                if hit.entering {
                    // if first_hit.is_none() || hit.distance < first_hit.as_ref().unwrap().distance {
                    if !first_hit
                        .as_ref()
                        .is_some_and(|h| h.distance > hit.distance)
                    {
                        first_hit = Some(hit);
                    }
                // } else if back_hit.is_none() || hit.distance > back_hit.as_ref().unwrap().distance {
                } else if !back_hit.as_ref().is_some_and(|h| h.distance < hit.distance) {
                    back_hit = Some(hit);
                }
            }
        }

        let mut hit_vec = hitvec![];
        if let Some(hit) = first_hit {
            hit_vec.push(hit);
        }
        if let Some(hit) = back_hit {
            hit_vec.push(hit);
        }

        hit_vec
    }

    fn apply_transform(&mut self, transform: &Transform) {
        self.corner.apply_transform(transform);
        self.planes = OnceLock::new();
    }
}
