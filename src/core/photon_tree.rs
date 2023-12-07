use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use kd_tree::KdTree3;

use super::{photon::Photon, vertex::Vertex};

pub struct PhotonTree {
    tree: KdTree3<Photon>,
}

impl PhotonTree {
    pub fn build(photons: Vec<Photon>) -> Self {
        let tree = KdTree3::build_by_ordered_float(photons);
        Self { tree }
    }

    pub fn get_within_radius(&self, position: &Vertex, radius: f32) -> Vec<PhotonAndDistance> {
        let mut vec: Vec<PhotonAndDistance> = self
            .tree
            .within_radius(&position.xyz(), radius)
            .into_iter()
            .map(|photon| {
                let squared_distance = (photon.position.vector() - position.vector()).len_sqrd();
                PhotonAndDistance {
                    item: photon,
                    squared_distance,
                }
            })
            .collect();

        vec.sort_unstable_by(|a, b| a.squared_distance.partial_cmp(&b.squared_distance).unwrap());
        vec
    }

    pub fn find_nearest(&self, position: &Vertex, n: usize) -> Vec<PhotonAndDistance> {
        self.tree.nearests(&position.xyz(), n)
    }

    pub fn get_n_within_radius(
        &self,
        position: &Vertex,
        radius: f32,
        n: usize,
    ) -> Vec<PhotonAndDistance> {
        let mut vec = self.get_within_radius(position, radius);
        vec.truncate(n);
        vec
    }
}

type PhotonAndDistance<'a> = kd_tree::ItemAndDistance<'a, Photon, f32>;
