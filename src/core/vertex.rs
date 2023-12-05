use super::{tex_coords::TexCoords, transform::Transform, vector::Vector};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn xyz(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub const fn vector(&self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }

    pub fn vector_to(&self, other: &Vertex) -> Vector {
        Vector::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    pub fn distance(&self, other: &Vertex) -> f32 {
        self.vector_to(other).length()
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        let matrix = &transform.matrix;

        let x =
            matrix[0][0] * self.x + matrix[0][1] * self.y + matrix[0][2] * self.z + matrix[0][3];
        let y =
            matrix[1][0] * self.x + matrix[1][1] * self.y + matrix[1][2] * self.z + matrix[1][3];
        let z =
            matrix[2][0] * self.x + matrix[2][1] * self.y + matrix[2][2] * self.z + matrix[2][3];
        let w =
            matrix[3][0] * self.x + matrix[3][1] * self.y + matrix[3][2] * self.z + matrix[3][3];

        self.x = x;
        self.y = y;
        self.z = z;
    }
}

impl std::ops::Add<Vector> for Vertex {
    type Output = Vertex;

    fn add(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
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

impl std::ops::Sub<Vector> for Vertex {
    type Output = Vertex;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl From<Vector> for Vertex {
    fn from(vector: Vector) -> Self {
        Self::new(vector.x, vector.y, vector.z)
    }
}

#[derive(Clone)]
pub struct RichVertex {
    pub vertex: Vertex,
    pub normal: Option<Vector>,
    pub tex_coords: Option<TexCoords>,
}

impl RichVertex {
    pub fn new(vertex: Vertex, normal: Option<Vector>, tex_coords: Option<TexCoords>) -> Self {
        Self {
            vertex,
            normal,
            tex_coords,
        }
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        self.vertex.apply_transform(transform);

        if let Some(normal) = &mut self.normal {
            normal.apply_transform(transform);
        }
    }
}

impl std::ops::Deref for RichVertex {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        &self.vertex
    }
}

impl From<Vertex> for RichVertex {
    fn from(vertex: Vertex) -> Self {
        Self::new(vertex, None, None)
    }
}
