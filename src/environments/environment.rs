use crate::{
    core::{
        colour::Colour,
        hit::{Hit, HitVec},
        ray::Ray,
    },
    lights::light::Light,
    objects::object::Object,
};

pub struct RaytraceResult {
    pub colour: Colour,
    pub depth: f32,
}

impl RaytraceResult {
    pub fn none() -> Self {
        Self {
            colour: Colour::black(),
            depth: 0.0,
        }
    }
}

pub trait Environment: Send + Sync {
    fn pre_render(&mut self);
    fn raytrace(&self, ray: &Ray) -> RaytraceResult;

    fn add_object(&mut self, object: Box<dyn Object + 'static>);
    fn add_light(&mut self, light: Box<dyn Light + 'static>);

    fn objects(&self) -> &[Box<dyn Object>];

    fn select_first_hit<'s>(&self, hits: HitVec<'s>) -> Option<Hit<'s>> {
        let mut min_hit: Option<Hit> = None;
        let mut min_distance = std::f32::MAX;

        for hit in hits {
            if hit.distance < 0.0 {
                continue;
            }
            if !hit.entering {
                continue;
            }

            if hit.distance < min_distance {
                min_distance = hit.distance;
                min_hit = Some(hit);
            }
        }

        min_hit
    }

    fn trace(&self, ray: &Ray) -> Option<Hit> {
        let mut min_hit: Option<Hit> = None;
        let mut min_distance = std::f32::MAX;

        for object in self.objects() {
            let hits = object.intersect(ray);

            let hit = self.select_first_hit(hits);
            let Some(hit) = hit else {
                continue;
            };

            if hit.distance < min_distance {
                min_distance = hit.distance;
                min_hit = Some(hit);
            }
        }

        min_hit
    }
}
