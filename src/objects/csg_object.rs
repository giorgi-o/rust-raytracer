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

pub struct Csg {
    pub mode: CsgMode,
    pub left: Box<dyn Object>,
    pub right: Box<dyn Object>,
}

impl Csg {
    pub fn new(mode: CsgMode, left: impl Object + 'static, right: impl Object + 'static) -> Self {
        Self {
            mode,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

impl Object for Csg {
    fn intersect(&self, ray: &Ray) -> HitVec {
        /* 	const CSG::Action actions[3][8] = { {CSG::CSG_ADROP, CSG_BDROP,CSG_AEXIT,CSG_BDROP,CSG_ADROP,CSG_BEXIT,CSG_AENTER,CSG_BENTER},
            {CSG::CSG_AEXIT, CSG_BEXIT,CSG_ADROP,CSG_BENTER,CSG_AENTER,CSG_BDROP,CSG_ADROP,CSG_BDROP},
        {CSG::CSG_ADROP, CSG_BENTER,CSG_AEXIT,CSG_BEXIT,CSG_ADROP,CSG_BDROP,CSG_AENTER,CSG_BDROP} }; */
        let actions = match self.mode {
            CsgMode::Union => [
                CsgAction::ADrop,
                CsgAction::BDrop,
                CsgAction::AExit,
                CsgAction::BDrop,
                CsgAction::ADrop,
                CsgAction::BExit,
                CsgAction::AEnter,
                CsgAction::BEnter,
            ],
            CsgMode::Intersection => [
                CsgAction::AExit,
                CsgAction::BExit,
                CsgAction::ADrop,
                CsgAction::BEnter,
                CsgAction::AEnter,
                CsgAction::BDrop,
                CsgAction::ADrop,
                CsgAction::BDrop,
            ],
            CsgMode::Difference => [
                CsgAction::ADrop,
                CsgAction::BEnter,
                CsgAction::AExit,
                CsgAction::BExit,
                CsgAction::ADrop,
                CsgAction::BDrop,
                CsgAction::AEnter,
                CsgAction::BDrop,
            ],
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
                CsgAction::AEnter | CsgAction::AExit => {
                    let mut left_hit = left_hits.next().unwrap();
                    left_hit.entering = actions[state] == CsgAction::AEnter;
                    hit_vec.push(left_hit);
                }
                CsgAction::ADrop => {
                    left_hits.next();
                }
                CsgAction::BEnter | CsgAction::BExit => {
                    let mut right_hit = right_hits.next().unwrap();
                    right_hit.entering = actions[state] == CsgAction::BEnter;
                    hit_vec.push(right_hit);
                }
                CsgAction::BDrop => {
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
