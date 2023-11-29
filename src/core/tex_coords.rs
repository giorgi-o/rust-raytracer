#[derive(Clone)]
pub struct TexCoords {
    pub u: f32,
    pub v: f32,
    pub s: f32,
    pub t: f32,
}

impl TexCoords {
    pub fn new(u: f32, v: f32, s: f32, t: f32) -> Self {
        Self { u, v, s, t }
    }
}