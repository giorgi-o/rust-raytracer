use std::sync::{RwLock, RwLockReadGuard};

use kiddo::{float::kdtree::KdTree, NearestNeighbour, SquaredEuclidean};

use super::{photon::Photon, vertex::Vertex};

pub struct PhotonTree {
    inner: RwLock<PhotonTreeInner>,
}

struct PhotonTreeInner {
    vec: Vec<Photon>,
    // tree: KdTree<f32, 3>,
    tree: KdTree<f32, u64, 3, 100_000, u32>,
}

impl PhotonTree {
    pub fn new() -> Self {
        let inner = PhotonTreeInner {
            vec: Vec::new(),
            tree: KdTree::new(),
        };
        Self {
            inner: RwLock::new(inner),
        }
    }

    pub fn insert(&self, photon: Photon) {
        let mut inner = self.inner.write().unwrap();
        let position = photon.position.xyz();
        inner.vec.push(photon);

        let index = inner.vec.len() - 1;
        inner.tree.add(&position, index as u64);
    }

    pub fn get_within_radius(&self, position: &Vertex, radius: f32) -> NeighbourPhotons<'_> {
        let inner_guard = self.inner.read().unwrap();
        let neighbours = inner_guard
            .tree
            .within::<SquaredEuclidean>(&position.xyz(), radius);

        NeighbourPhotons {
            inner_guard,
            neighbours,
        }
    }

    pub fn find_nearest(&self, position: &Vertex, n: usize) -> NeighbourPhotons<'_> {
        let inner_guard = self.inner.read().unwrap();
        let neighbours = inner_guard
            .tree
            .nearest_n::<SquaredEuclidean>(&position.xyz(), n);

        NeighbourPhotons {
            inner_guard,
            neighbours,
        }
    }
}

pub struct NeighbourPhotons<'a> {
    inner_guard: RwLockReadGuard<'a, PhotonTreeInner>,
    neighbours: Vec<NearestNeighbour<f32, u64>>,
}

impl NeighbourPhotons<'_> {
    pub fn iter(&self) -> impl Iterator<Item = &Photon> {
        self.neighbours
            .iter()
            .map(move |neighbour| &self.inner_guard.vec[neighbour.item as usize])
    }

    pub fn len(&self) -> usize {
        self.neighbours.len()
    }
}
