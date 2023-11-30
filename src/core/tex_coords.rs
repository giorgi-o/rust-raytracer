#[derive(Clone)]
pub struct TexCoords {
    pub u: f32,
    pub v: f32,
}

impl TexCoords {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

impl From<(f32, f32)> for TexCoords {
    fn from(coords: (f32, f32)) -> Self {
        Self::new(coords.0, coords.1)
    }
}
