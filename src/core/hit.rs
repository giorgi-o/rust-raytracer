use crate::{materials::material::Material, objects::object::Object};

use super::{tex_coords::TexCoords, vector::Vector, vertex::Vertex};

pub struct Hit<'s> {
    pub distance: f32,                 // intersection distance
    pub entering: bool,                // true if entering object, false if exiting
    pub what: &'s dyn Object,          // object that was hit
    pub material: &'s dyn Material,    // material at the point that was hit
    pub position: Vertex,              // position of intersection
    pub normal: Vector,                // normal at intersection
    pub tex_coords: Option<TexCoords>, // texture coordinates at intersection
}

impl<'s> Hit<'s> {
    pub const fn new(
        what: &'s dyn Object,
        entering: bool,
        distance: f32,
        position: Vertex,
        normal: Vector,
        material: &'s dyn Material,
        tex_coords: Option<TexCoords>,
    ) -> Self {
        Self {
            distance,
            entering,
            what,
            material,
            position,
            normal,
            tex_coords,
        }
    }

    // for hits that are infinitely far away
    pub const fn infinity(
        what: &'s dyn Object,
        entering: bool,
        distance: f32, // ∞ or -∞
        material: &'s dyn Material,
    ) -> Self {
        Self {
            distance,
            entering,
            what,
            material,
            position: Vertex::zero(),
            normal: Vector::zero(),
            tex_coords: None,
        }
    }
}

// stack allocated vector of at most N hits
const HITVEC_SIZE: u8 = 6;
pub struct HitVec<'s> {
    hits: [Option<Hit<'s>>; HITVEC_SIZE as usize],
    len: u8,
}

impl<'s> HitVec<'s> {
    pub const fn new() -> Self {
        Self {
            hits: [None, None, None, None, None, None],
            len: 0,
        }
    }

    pub fn push(&mut self, hit: Hit<'s>) {
        if self.len >= HITVEC_SIZE {
            panic!("HitVec overflow");
        }

        self.hits[self.len as usize] = Some(hit);
        self.len += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Hit<'_>> {
        self.hits[..self.len as usize]
            .iter()
            .map(|hit| hit.as_ref().unwrap())
    }

    pub fn iter_mut(&'s mut self) -> impl Iterator<Item = &mut Hit<'_>> {
        self.hits[..self.len as usize]
            .iter_mut()
            .map(move |hit| hit.as_mut().unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// hitvec![hit1, ...] macro
#[macro_export]
macro_rules! hitvec {
    () => {
        HitVec::new()
    };
    ($($x:expr),+ $(,)?) => {
        {
            let mut hitvec = HitVec::new();
            $(
                hitvec.push($x);
            )+
            hitvec
        }
    };
}

pub struct HitVecIntoIter<'s> {
    hitvec: HitVec<'s>,
    index: u8,
}

impl<'s> Iterator for HitVecIntoIter<'s> {
    type Item = Hit<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.hitvec.len {
            None
        } else {
            let hit = self.hitvec.hits[self.index as usize].take().unwrap();
            self.index += 1;
            Some(hit)
        }
    }
}

impl<'s> IntoIterator for HitVec<'s> {
    type Item = Hit<'s>;
    type IntoIter = HitVecIntoIter<'s>;

    fn into_iter(self) -> Self::IntoIter {
        HitVecIntoIter {
            hitvec: self,
            index: 0,
        }
    }
}


/*
// alternate implementation
use std::cell::RefCell;

pub struct HitVec<'s> {
    vec: Vec<Hit<'s>>,
}

thread_local! {
    static POOL: RefCell<Vec<Vec<Hit<'static>>>> = RefCell::new(Vec::new());
}

impl<'s> HitVec<'s> {
    #[inline(never)]
    pub fn new() -> Self {
        POOL.with(|pool| {
            // let pool = &mut *pool.borrow_mut();
            // let vec = pool.pop().unwrap_or_default();
            let vec = match pool.borrow_mut().pop() {
                Some(vec) => unsafe { std::mem::transmute(vec) },
                None => Vec::with_capacity(4),
            };
            HitVec { vec }
        })
    }

    #[inline(never)]
    pub fn push(&mut self, hit: Hit<'s>) {
        // println!("Vec capacity: {}/{}", self.vec.len(), self.vec.capacity());
        self.vec.push(hit);
    }

    #[inline(never)]
    pub fn iter(&self) -> impl Iterator<Item = &Hit<'s>> {
        self.vec.iter()
    }

    #[inline(never)]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl Drop for HitVec<'_> {
    #[inline(never)]
    fn drop(&mut self) {
        let mut vec: Vec<Hit<'_>> = std::mem::take(&mut self.vec);
        vec.clear();

        let vec: Vec<Hit<'static>> = unsafe { std::mem::transmute(vec) };
        POOL.with(|pool| {
            pool.borrow_mut().push(vec);
        })
    }
}

pub struct HitVecIntoIter<'s> {
    hitvec: HitVec<'s>,
    index: usize,
}

impl<'s> Iterator for HitVecIntoIter<'s> {
    type Item = Hit<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.hitvec.vec.len() {
            None
        } else {
            let hit = self.hitvec.vec[self.index].clone();
            self.index += 1;
            Some(hit)
        }
    }
}

impl<'s> IntoIterator for HitVec<'s> {
    type Item = Hit<'s>;
    type IntoIter = HitVecIntoIter<'s>;

    fn into_iter(self) -> Self::IntoIter {
        HitVecIntoIter {
            hitvec: self,
            index: 0,
        }
    }
}

// hitvec![hit1, ...] macro
#[macro_export]
macro_rules! hitvec {
    () => {
        HitVec::new()
    };
    ($($x:expr),+ $(,)?) => {
        {
            let mut hitvec = HitVec::new();
            $(
                hitvec.push($x);
            )+
            hitvec
        }
    };
}
*/