use super::{transform::Transform, vertex::Vertex};

#[derive(Debug, Copy, Clone)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn random() -> Self {
        use rand::distributions::Distribution;

        let mut rng = rand::thread_rng();
        thread_local! {
            static DISTRIBUTION: rand::distributions::Uniform<f32> = rand::distributions::Uniform::new(-1.0, 1.0);
        }

        DISTRIBUTION.with(|distribution| loop {
            let vec = Self::new(
                distribution.sample(&mut rng),
                distribution.sample(&mut rng),
                distribution.sample(&mut rng),
            );

            if vec.len_sqrd() <= 1.0 {
                return vec.normalised();
            }
        })
    }

    pub fn random_on_surface(normal: Vector) -> Self {
        let mut vec = Self::random();

        if vec.dot(&normal) < 0.0 {
            vec.negate();
        }

        vec
    }

    pub fn normalise(&mut self) {
        let length = self.length();
        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    pub fn normalised(&self) -> Self {
        let length = self.length();
        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    pub fn len_sqrd(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.len_sqrd().sqrt()
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn reflection(&self, initial: &Self) -> Self {
        let d = self.dot(initial) * 2.0;

        Self {
            x: initial.x - d * self.x,
            y: initial.y - d * self.y,
            z: initial.z - d * self.z,
        }
    }

    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    pub fn negated(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn angle(&self, other: &Self) -> f32 {
        let dot = self.dot(other);
        let len = self.length() * other.length();

        dot.acos() / len
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        let matrix = &transform.matrix;

        let x = matrix[0][0] * self.x + matrix[0][1] * self.y + matrix[0][2] * self.z;
        let y = matrix[1][0] * self.x + matrix[1][1] * self.y + matrix[1][2] * self.z;
        let z = matrix[2][0] * self.x + matrix[2][1] * self.y + matrix[2][2] * self.z;

        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn to_tangent_space(mut self, tangent: &Self, normal: &Self) -> Self {
        let tangent = tangent.normalised();
        let normal = normal.normalised();
        let bitangent = normal.cross(&tangent);

        let rotation_matrix = Transform::from_rotation_matrix([
            [tangent.x, bitangent.x, normal.x],
            [tangent.y, bitangent.y, normal.y],
            [tangent.z, bitangent.z, normal.z],
        ]);

        self.apply_transform(&rotation_matrix);
        self.normalised()
    }
}

impl std::ops::Mul<Vector> for Vector {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs as f32,
            y: self.y * rhs as f32,
            z: self.z * rhs as f32,
        }
    }
}

impl std::ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::AddAssign<Vector> for Vector {
    fn add_assign(&mut self, rhs: Vector) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl From<Vertex> for Vector {
    fn from(vertex: Vertex) -> Self {
        Self {
            x: vertex.x,
            y: vertex.y,
            z: vertex.z,
        }
    }
}

// vector with spherical coordinates
#[derive(Debug, Copy, Clone)]
pub struct SVector {
    pub r: f32,     // radius
    pub alpha: f32, // angle from X to Z axis
    pub beta: f32,  // angle from Z to Y axis
}

impl SVector {
    pub fn new(r: f32, alpha: f32, beta: f32) -> Self {
        Self { r, alpha, beta }
    }
}

impl From<Vector> for SVector {
    fn from(vector: Vector) -> Self {
        let r = vector.length();
        let alpha = vector.z.atan2(vector.x);
        let beta = vector.y.atan2(vector.x.hypot(vector.z));

        Self { r, alpha, beta }
    }
}

impl From<SVector> for Vector {
    fn from(svector: SVector) -> Self {
        Self {
            x: svector.r * svector.alpha.cos() * svector.beta.cos(),
            y: svector.r * svector.beta.sin(),
            z: svector.r * svector.alpha.sin() * svector.beta.cos(),
        }
    }
}
