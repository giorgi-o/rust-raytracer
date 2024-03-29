#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Colour {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub const fn grey(intensity: f32) -> Self {
        Self::new(intensity, intensity, intensity)
    }

    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn is_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
    }

    pub fn scale(&mut self, scalar: f32) {
        self.r *= scalar;
        self.g *= scalar;
        self.b *= scalar;
    }

    pub fn scaled(&self, scalar: f32) -> Self {
        Self::new(self.r * scalar, self.g * scalar, self.b * scalar)
    }
}

impl std::ops::Mul<Colour> for Colour {
    type Output = Colour;

    fn mul(self, rhs: Colour) -> Self::Output {
        Self::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
        )
    }
}

impl std::ops::Add<Colour> for Colour {
    type Output = Colour;

    fn add(self, rhs: Colour) -> Self::Output {
        Self::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
        )
    }
}

impl std::ops::Mul<f32> for Colour {
    type Output = Colour;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl std::ops::Div<f32> for Colour {
    type Output = Colour;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl std::ops::AddAssign<Colour> for Colour {
    fn add_assign(&mut self, rhs: Colour) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}
