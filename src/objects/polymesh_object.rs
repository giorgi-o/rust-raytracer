use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
};

use crate::{
    core::{
        hit::{Hit, HitVec},
        ray::Ray,
        transform::Transform,
        vector::Vector,
        vertex::{RichVertex, Vertex},
    },
    hitvec,
    materials::material::Material,
};

use super::{object::Object, triangle_object::Triangle};

pub struct PolyMesh {
    vertices: Vec<RichVertex>,
    triangles: Vec<Triangle>,
    normals: Vec<Vector>,
    smooth: bool,
    material: Arc<dyn Material>,

    // map from vertex index to indexes of adjacent triangles
    vertex_to_triangles: HashMap<usize, Vec<usize>>,
}

impl PolyMesh {
    pub fn from_obj_file(
        path: std::path::PathBuf,
        material: Arc<dyn Material>,
        smooth: bool,
    ) -> Self {
        let obj_file = File::open(path.clone()).unwrap_or_else(|e| {
            panic!(
                "Could not open OBJ file at path {} (cwd: {:?})\n{}",
                path.to_str().unwrap(),
                std::env::current_dir().unwrap(),
                e
            )
        });

        let mut this = Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            normals: Vec::new(),
            smooth,
            material,
            vertex_to_triangles: HashMap::new(),
        };

        let reader = BufReader::new(obj_file);
        for line in reader.lines() {
            let line = line.expect("Could not read next line from OBJ file");

            if line.is_empty() {
                continue;
            }

            let words: Vec<&str> = line.split_whitespace().collect();
            match words[0] {
                "#" => continue,
                "v" => {
                    let x = words[1]
                        .parse::<f32>()
                        .expect("Could not parse vertex x coordinate");
                    let y = words[2]
                        .parse::<f32>()
                        .expect("Could not parse vertex y coordinate");
                    let z = words[3]
                        .parse::<f32>()
                        .expect("Could not parse vertex z coordinate");
                    this.vertices.push(Vertex::new(x, y, z).into());
                }
                "vn" => {
                    let x = words[1]
                        .parse::<f32>()
                        .expect("Could not parse normal x coordinate");
                    let y = words[2]
                        .parse::<f32>()
                        .expect("Could not parse normal y coordinate");
                    let z = words[3]
                        .parse::<f32>()
                        .expect("Could not parse normal z coordinate");
                    this.normals.push(Vector::new(x, y, z));
                }
                "f" => {
                    this.parse_face(words);
                }
                _ => {}
            }
        }

        // at this point, all the faces have been parsed. go through them again
        // and calculate any missing vertex normals.
        if smooth {
            for vertex_index in 0..this.vertices.len() {
                let vertex = &this.vertices[vertex_index];
                if vertex.normal.is_some() {
                    continue;
                }

                this.calculate_normal(vertex_index);

                // go to all the triangles that have this vertex to give them
                // the new vertex normal
                for triangle_index in this.vertex_to_triangles[&vertex_index].iter() {
                    // triangle_index is the index of the triangle that has
                    // this vertex as one of its points

                    let vertex = &this.vertices[vertex_index];
                    let triangle = &mut this.triangles[*triangle_index];
                    let (a_index, b_index, c_index) = triangle.vertex_indices;

                    if a_index == vertex_index {
                        triangle.a.normal = vertex.normal;
                    } else if b_index == vertex_index {
                        triangle.b.normal = vertex.normal;
                    } else if c_index == vertex_index {
                        triangle.c.normal = vertex.normal;
                    }
                }
            }
        }

        this
    }

    fn parse_face(&mut self, words: Vec<&str>) {
        // the line is of the form:
        // f 1/2/3 4/5/6 7/8/9 [10/11/12]

        // vec of (vertex index, optional[normal index])
        let mut indices_in_obj: Vec<(usize, Option<usize>)> = Vec::new();

        for vertex_info in words.iter().skip(1) {
            let numbers: Vec<&str> = vertex_info.split('/').collect();

            let vertex_index = numbers[0]
                .parse::<usize>()
                .expect("Could not parse vertex index")
                - 1;
            let normal_index = numbers
                .get(2)
                .map(|n| n.parse::<usize>().expect("Could not parse normal index") - 1);

            indices_in_obj.push((vertex_index, normal_index));
        }

        // function to create, process and store a triangle
        let mut create_triangle = |i: usize, j: usize, k: usize| {
            // i, j, k are the indices of indices_in_obj

            // (index in vertices, index in normals)
            let (av, an) = indices_in_obj[i];
            let (bv, bn) = indices_in_obj[j];
            let (cv, cn) = indices_in_obj[k];

            // set normals
            let get_normal = |index: usize| self.normals[index];
            self.vertices[av].normal = an.map(get_normal);
            self.vertices[bv].normal = bn.map(get_normal);
            self.vertices[cv].normal = cn.map(get_normal);

            let triangle = Triangle::new(
                self.vertices[av].clone(),
                self.vertices[bv].clone(),
                self.vertices[cv].clone(),
                (av, bv, cv),
                self.material.clone(),
                self.smooth,
            );
            self.triangles.push(triangle);

            // add the triangle to the vertex_to_triangles map
            for index in [i, j, k] {
                let entry = self.vertex_to_triangles.entry(index).or_default();
                entry.push(self.triangles.len() - 1);
            }
        };

        // create first triangle
        create_triangle(0, 1, 2);

        // if there's a fourth vertex, create second triangle
        if indices_in_obj.len() == 4 {
            create_triangle(0, 2, 3);
        }
    }

    fn calculate_normal(&mut self, vertex_index: usize) {
        let vertex = &mut self.vertices[vertex_index];
        if vertex.normal.is_some() {
            return;
        }

        // average the normals of the triangles adjacent to this vertex
        let mut average_normal = Vector::zero();
        for triangle_index in self.vertex_to_triangles[&vertex_index].iter() {
            let triangle = &self.triangles[*triangle_index];
            let triangle_normal = triangle.get_plane_normal();
            average_normal += triangle_normal;
        }

        average_normal.normalise();
        vertex.normal = Some(average_normal);
    }
}

impl Object for PolyMesh {
    fn intersect(&self, ray: &Ray) -> HitVec {
        let mut closest_hit: Option<Hit> = None;

        for triangle in self.triangles.iter() {
            let triangle_hits = triangle.intersect(ray);
            for triangle_hit in triangle_hits {
                if triangle_hit.distance < 0.0 {
                    continue;
                }

                // if our hit is closer, it becomes the new closest hit
                match closest_hit {
                    None => closest_hit = Some(triangle_hit),
                    Some(ref hit) => {
                        if triangle_hit.distance < hit.distance {
                            closest_hit = Some(triangle_hit);
                        }
                    }
                }
            }
        }

        match closest_hit {
            None => hitvec![],
            Some(hit) => hitvec![hit],
        }
    }

    fn apply_transform(&mut self, transform: &Transform) {
        for vertex in self.vertices.iter_mut() {
            vertex.apply_transform(transform);
        }

        for triangle in self.triangles.iter_mut() {
            triangle.apply_transform(transform);
        }
    }
}
