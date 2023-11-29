use super::{transform::Transform, vector::Vector};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub normal: Option<Vector>,
}

impl Vertex {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x,
            y,
            z,
            w,
            normal: None,
        }
    }

    pub const fn new_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 1.0)
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub const fn vector(&self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        let matrix = &transform.matrix;

        let x = matrix[0][0] * self.x
            + matrix[0][1] * self.y
            + matrix[0][2] * self.z
            + matrix[0][3] * self.w;
        let y = matrix[1][0] * self.x
            + matrix[1][1] * self.y
            + matrix[1][2] * self.z
            + matrix[1][3] * self.w;
        let z = matrix[2][0] * self.x
            + matrix[2][1] * self.y
            + matrix[2][2] * self.z
            + matrix[2][3] * self.w;
        let w = matrix[3][0] * self.x
            + matrix[3][1] * self.y
            + matrix[3][2] * self.z
            + matrix[3][3] * self.w;

        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;

        if let Some(normal) = &mut self.normal {
            normal.apply_transform(transform);
        }
    }
}

impl std::ops::Add<Vector> for Vertex {
    type Output = Vertex;

    fn add(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w,
            normal: None,
        }
    }
}

impl std::ops::AddAssign<Vector> for Vertex {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl From<Vector> for Vertex {
    fn from(vector: Vector) -> Self {
        Self::new_xyz(vector.x, vector.y, vector.z)
    }
}
