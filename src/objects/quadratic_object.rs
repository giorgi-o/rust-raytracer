use std::sync::Arc;

use crate::{
    core::{
        hit::{Hit, HitVec},
        ray::Ray,
        transform::Transform,
        vector::Vector,
    },
    hitvec,
    materials::material::Material,
};

use super::object::Object;

pub struct Quadratic {
    // the variables are the coefficients of the quadratic equation
    // ax^2 + 2bxy + 2cxz + 2dx + ey^2 + 2fyz + 2gy + hz^2 + 2iz + j = 0
    variables: (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32),
    material: Arc<dyn Material>,
}

impl Quadratic {
    pub fn new(
        variables: (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32),
        material: Arc<dyn Material>,
    ) -> Box<Self> {
        Box::new(Self {
            variables,
            material,
        })
    }

    pub fn cylinder(diameter: f32, material: Arc<dyn Material>) -> Box<Self> {
        let radius = diameter / 2.;
        let a = 1. / radius.powi(2);
        Self::new((a, 0., 0., 0., 0., 0., 0., a, 0., -1.), material)
    }
}

impl Object for Quadratic {
    #[allow(non_snake_case)]
    fn intersect(&self, ray: &Ray) -> HitVec {
        let P = ray.position.clone();
        let D = ray.direction;
        let (a, b, c, d, e, f, g, h, i, j) = self.variables;

        let Aq = a * D.x.powi(2)
            + 2.0 * b * D.x * D.y
            + 2.0 * c * D.x * D.z
            + e * D.y.powi(2)
            + 2.0 * f * D.y * D.z
            + h * D.z.powi(2);
        let Bq = 2.0
            * (a * P.x * D.x
                + b * (P.x * D.y + P.y * D.x)
                + c * (P.x * D.z + D.x * P.z)
                + d * D.x
                + e * P.y * D.y
                + f * (P.y * D.z + D.y * P.z)
                + g * D.y
                + h * P.z * D.z
                + i * D.z);
        let Cq = a * P.x.powi(2)
            + 2.0 * b * P.x * P.y
            + 2.0 * c * P.x * P.z
            + 2.0 * d * P.x
            + e * P.y.powi(2)
            + 2.0 * f * P.y * P.z
            + 2.0 * g * P.y
            + h * P.z.powi(2)
            + 2.0 * i * P.z
            + j;

        if Aq.abs() == 0.0 {
            // only one tangent intersection, return nothing
            return hitvec![];
        }

        let discriminant = Bq.powi(2) - 4.0 * Aq * Cq;
        if discriminant < 0.0 {
            // no intersection
            return hitvec![];
        }

        let discriminant = discriminant.sqrt();
        let t0 = (-Bq - discriminant) / (2.0 * Aq);
        let t1 = (-Bq + discriminant) / (2.0 * Aq);
        let (t0, t1) = if t0 > t1 { (t1, t0) } else { (t0, t1) };

        let create_hit = |t: f32, entering| {
            let position = P.clone() + D * t;
            let Vector {
                x: xi,
                y: yi,
                z: zi,
            } = position.vector();

            let mut normal = Vector::new(
                a * xi + b * yi + c * zi + d,
                b * xi + e * yi + f * zi + g,
                c * xi + f * yi + h * zi + i,
            )
            .normalised();
            if normal.dot(&D) > 0.0 {
                normal.negate();
            }

            Hit::new(
                self,
                entering,
                t,
                position,
                normal,
                self.material.as_ref(),
                None,
            )
        };

        let hit1 = create_hit(t0, true);
        let hit2 = create_hit(t1, false);

        hitvec![hit1, hit2]
    }

    #[allow(non_snake_case)]
    fn apply_transform(&mut self, T: &Transform) {
        let (a, b, c, d, e, f, g, h, i, j) = self.variables;
        let Q = Transform::from_matrix([[a, b, c, d], [b, e, f, g], [c, f, h, i], [d, g, i, j]]);

        let T = T.clone().inverse();
        let T_T = T.transposed();

        let Q = T_T * Q * T;

        let a = Q.matrix[0][0];
        let b = Q.matrix[0][1];
        let c = Q.matrix[0][2];
        let d = Q.matrix[0][3];
        let e = Q.matrix[1][1];
        let f = Q.matrix[1][2];
        let g = Q.matrix[1][3];
        let h = Q.matrix[2][2];
        let i = Q.matrix[2][3];
        let j = Q.matrix[3][3];

        self.variables = (a, b, c, d, e, f, g, h, i, j);
    }
}
