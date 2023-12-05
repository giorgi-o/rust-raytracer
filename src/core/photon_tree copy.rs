use std::sync::{RwLock, RwLockReadGuard};

use kiddo::{KdTree, NearestNeighbour, SquaredEuclidean, ImmutableKdTree};

use super::{photon::Photon, vertex::Vertex};

pub struct PhotonTree {
    vec: Vec<Photon>,
    tree: Option<ImmutableKdTree<f32, 3>>,
}

impl PhotonTree {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            tree: None,
        }
    }

    pub fn insert(&mut self, photon: Photon) {
        self.vec.push(photon);
    }

    pub fn build(&mut self) {
        let mut tree = KdTree::new_with_capacity(3, self.vec.len());
        for (i, photon) in self.vec.iter().enumerate() {
            tree.add([photon.position.x, photon.position.y, photon.position.z], i as u64)
                .unwrap();
        }
        self.tree = Some(tree.into_immutable());
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
}
