use std::{
    collections::HashMap,
    sync::{RwLock, RwLockReadGuard},
};

use kd_tree::KdTree3;

use super::{photon::Photon, vertex::Vertex};

struct PhotonTreeInConstruction {
    photons: Vec<Photon>,
}

struct BuiltPhotonTree {
    tree: KdTree3<Photon>,
}

pub struct PhotonTree(PhotonTreeState);

enum PhotonTreeState {
    InConstruction(PhotonTreeInConstruction),
    Built(BuiltPhotonTree),
}

impl PhotonTree {
    pub fn new() -> Self {
        let tree = PhotonTreeInConstruction {
            photons: Vec::new(),
        };
        Self(PhotonTreeState::InConstruction(tree))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let tree = PhotonTreeInConstruction {
            photons: Vec::with_capacity(capacity),
        };
        Self(PhotonTreeState::InConstruction(tree))
    }

    fn in_construction(&mut self) -> &mut PhotonTreeInConstruction {
        match &mut self.0 {
            PhotonTreeState::InConstruction(tree) => tree,
            _ => panic!("Photon tree is not in construction"),
        }
    }

    fn built(&self) -> &BuiltPhotonTree {
        match &self.0 {
            PhotonTreeState::Built(tree) => tree,
            _ => panic!("Photon tree is not built"),
        }
    }

    pub fn insert(&mut self, photon: Photon) {
        self.in_construction().photons.push(photon);
    }

    pub fn build(&mut self) {
        let photons = std::mem::take(&mut self.in_construction().photons);
        let tree = KdTree3::build_by_ordered_float(photons);
        let tree = BuiltPhotonTree { tree };
        self.0 = PhotonTreeState::Built(tree);
    }

    pub fn get_within_radius(&self, position: &Vertex, radius: f32) -> Vec<PhotonAndDistance> {
        self.built()
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
            .collect()
    }

    pub fn find_nearest(&self, position: &Vertex, n: usize) -> Vec<PhotonAndDistance> {
        self.built().tree.nearests(&position.xyz(), n)
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
