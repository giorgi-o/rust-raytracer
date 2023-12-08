use std::sync::{Arc, OnceLock};

use crate::{
    core::{
        hit::{Hit, HitVec},
        ray::Ray,
        transform::Transform,
        vector::Vector,
        vertex::RichVertex,
    },
    hitvec,
    materials::material::Material,
};

use super::{object::Object, plane_object::Plane};

struct Barycentric {
    // alpha is the weight of vertex a, etc.
    alpha: f32,
    beta: f32,
    gamma: f32,
}

pub struct Triangle {
    pub a: RichVertex,
    pub b: RichVertex,
    pub c: RichVertex,
    pub ab: Vector,
    pub bc: Vector,
    pub ca: Vector,

    smooth: bool,
    material: Arc<dyn Material>,

    plane: OnceLock<Plane>,
    plane_normal: OnceLock<Vector>,

    // the index of a, b, c in the polymesh's vertex list
    pub vertex_indices: (usize, usize, usize),
}

impl Triangle {
    pub fn new(
        a: RichVertex,
        b: RichVertex,
        c: RichVertex,
        vertex_indices: (usize, usize, usize),
        material: Arc<dyn Material>,
        smooth: bool,
    ) -> Self {
        let ab = b.vector() - a.vector();
        let bc = c.vector() - b.vector();
        let ca = a.vector() - c.vector();

        Self {
            a,
            b,
            c,
            ab,
            bc,
            ca,
            smooth,
            material,
            plane: OnceLock::new(),
            plane_normal: OnceLock::new(),
            vertex_indices,
        }
    }

    pub fn set_smooth(&mut self, smooth: bool) {
        self.smooth = smooth;
    }

    pub fn set_vertex_normals(
        &mut self,
        an: Option<Vector>,
        bn: Option<Vector>,
        cn: Option<Vector>,
    ) {
        self.a.normal = an;
        self.b.normal = bn;
        self.c.normal = cn;
    }

    pub fn get_plane_normal(&self) -> Vector {
        *self
            .plane_normal
            .get_or_init(|| self.ab.cross(&self.bc).normalised())
    }

    fn get_plane(&self) -> &Plane {
        self.plane.get_or_init(|| {
            let plane_normal = self.get_plane_normal();
            Plane::new_raw(&self.a, self.ab, plane_normal, self.material.clone())
        })
    }

    fn get_barycentric(&self, ap: &Vector, bp: &Vector, cp: &Vector) -> Barycentric {
        // note: these are not actually the area, to get it we would divide by 2.
        // but since we're normalising the hit normal anyway we can skip it.
        let abp_area_x2 = ap.cross(&self.ab).length() /* / 2 */;
        let bcp_area_x2 = bp.cross(&self.bc).length() /* / 2 */;
        let cap_area_x2 = cp.cross(&self.ca).length() /* / 2 */;

        // normally we would divide alpha/beta/gamma by the total area to get actual
        // barycentric coordinates, but see above for why we don't need to.
        // float total_area = abi_area + bci_area + cai_area;

        Barycentric {
            alpha: bcp_area_x2, /* / total_area */
            beta: cap_area_x2,  /* / total_area */
            gamma: abp_area_x2, /* / total_area */
        }
    }

    fn smoothen_hit(&self, hit: &mut Hit, ai: &Vector, bi: &Vector, ci: &Vector) {
        let normal_none_err_msg = "Vertex normals not set in smoothen_hit()";
        let an = *self.a.normal.as_ref().expect(normal_none_err_msg);
        let bn = *self.b.normal.as_ref().expect(normal_none_err_msg);
        let cn = *self.c.normal.as_ref().expect(normal_none_err_msg);

        let barycentric = self.get_barycentric(ai, bi, ci);
        let normal = an * barycentric.alpha + bn * barycentric.beta + cn * barycentric.gamma;
        hit.normal = normal.normalised();
    }
}

impl Object for Triangle {
    fn intersect(&self, ray: &Ray) -> HitVec {
        let plane = self.get_plane();
        let plane_hits = plane.intersect(ray);
        let mut triangle_hits = hitvec![];

        for mut plane_hit in plane_hits {
            let intersection_point = &plane_hit.position;

            let ai = intersection_point.vector() - self.a.vector();
            let bi = intersection_point.vector() - self.b.vector();
            let ci = intersection_point.vector() - self.c.vector();

            // check if the normals are all in the same direction
            let ab_normal = ai.cross(&self.ab);
            let bc_normal = bi.cross(&self.bc);
            let ca_normal = ci.cross(&self.ca);

            let intersects_with_triangle =
                ab_normal.dot(&bc_normal) > 0.0 && bc_normal.dot(&ca_normal) > 0.0;
            if !intersects_with_triangle {
                continue;
            }

            if self.smooth {
                self.smoothen_hit(&mut plane_hit, &ai, &bi, &ci);
            }

            triangle_hits.push(plane_hit);
        }

        triangle_hits
    }

    fn apply_transform(&mut self, transform: &Transform) {
        self.a.apply_transform(transform);
        self.b.apply_transform(transform);
        self.c.apply_transform(transform);

        self.ab = self.b.vector() - self.a.vector();
        self.bc = self.c.vector() - self.b.vector();
        self.ca = self.a.vector() - self.c.vector();

        self.plane_normal = OnceLock::new();
        self.plane = OnceLock::new();
    }
}
