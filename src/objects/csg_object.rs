use crate::{
    core::{hit::HitVec, ray::Ray, transform::Transform},
    hitvec,
};

use super::object::Object;

pub enum CsgMode {
    Union,
    Intersection,
    Difference,
}

#[derive(PartialEq, Eq)]
pub enum CsgAction {
    AEnter, // ray enters A
    AExit,  // ray exits A
    ADrop,  // ray is dropped by A
    BEnter, // ray enters B
    BExit,  // ray exits B
    BDrop,  // ray is dropped by B
}
use CsgAction::*;

pub struct Csg {
    pub mode: CsgMode,
    pub left: Box<dyn Object>,
    pub right: Box<dyn Object>,
}

impl Csg {
    pub fn new(mode: CsgMode, left: Box<dyn Object>, right: Box<dyn Object>) -> Box<Self> {
        let this = Self { mode, left, right };
        Box::new(this)
    }
}

impl Object for Csg {
    fn intersect(&self, ray: &Ray) -> HitVec {
        let actions = match self.mode {
            CsgMode::Union => [ADrop, BDrop, AExit, BDrop, ADrop, BExit, AEnter, BEnter],
            CsgMode::Intersection => [AExit, BExit, ADrop, BEnter, AEnter, BDrop, ADrop, BDrop],
            CsgMode::Difference => [ADrop, BEnter, AExit, BExit, ADrop, BDrop, AEnter, BDrop],
        };

        let mut left_hits = self.left.intersect(ray).into_iter().peekable();
        let mut right_hits = self.right.intersect(ray).into_iter().peekable();

        let mut hit_vec = hitvec![];

        loop {
            let Some(left_hit) = left_hits.peek() else {
                break;
            };
            let Some(right_hit) = right_hits.peek() else {
                break;
            };

            let mut state = 0;
            if left_hit.entering {
                state += 4;
            }
            if right_hit.entering {
                state += 2;
            }
            if left_hit.distance > right_hit.distance {
                state += 1;
            }

            match actions[state] {
                AEnter | AExit => {
                    let mut left_hit = left_hits.next().unwrap();
                    left_hit.entering = actions[state] == AEnter;
                    hit_vec.push(left_hit);
                }
                ADrop => {
                    left_hits.next();
                }
                BEnter | BExit => {
                    let mut right_hit = right_hits.next().unwrap();
                    right_hit.entering = actions[state] == BEnter;
                    hit_vec.push(right_hit);
                }
                BDrop => {
                    right_hits.next();
                }
            }
        }

        match self.mode {
            CsgMode::Difference => {
                for left_hit in left_hits {
                    hit_vec.push(left_hit);
                }
            }
            CsgMode::Union => {
                for left_hit in left_hits {
                    hit_vec.push(left_hit);
                }
                for right_hit in right_hits {
                    hit_vec.push(right_hit);
                }
            }
            CsgMode::Intersection => {}
        }

        hit_vec
    }

    fn apply_transform(&mut self, transform: &Transform) {
        self.left.apply_transform(transform);
        self.right.apply_transform(transform);
    }
}
