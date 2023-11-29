#[derive(Clone)]
pub struct Transform {
    pub matrix: [[f32; 4]; 4],
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn from_matrix(matrix: [[f32; 4]; 4]) -> Self {
        Self { matrix }
    }

    pub fn from_rotation_matrix(matrix: [[f32; 3]; 3]) -> Self {
        Self {
            matrix: [
                [matrix[0][0], matrix[0][1], matrix[0][2], 0.0],
                [matrix[1][0], matrix[1][1], matrix[1][2], 0.0],
                [matrix[2][0], matrix[2][1], matrix[2][2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn inverse(&self) -> Self {
        let mut inverted: [[f32; 4]; 4] = [[0.0; 4]; 4];

        inverted[0][0] = self.matrix[1][1] * self.matrix[2][2] * self.matrix[3][3]
            - self.matrix[1][1] * self.matrix[2][3] * self.matrix[3][2]
            - self.matrix[2][1] * self.matrix[1][2] * self.matrix[3][3]
            + self.matrix[2][1] * self.matrix[1][3] * self.matrix[3][2]
            + self.matrix[3][1] * self.matrix[1][2] * self.matrix[2][3]
            - self.matrix[3][1] * self.matrix[1][3] * self.matrix[2][2];

        inverted[1][0] = -self.matrix[1][0] * self.matrix[2][2] * self.matrix[3][3]
            + self.matrix[1][0] * self.matrix[2][3] * self.matrix[3][2]
            + self.matrix[2][0] * self.matrix[1][2] * self.matrix[3][3]
            - self.matrix[2][0] * self.matrix[1][3] * self.matrix[3][2]
            - self.matrix[3][0] * self.matrix[1][2] * self.matrix[2][3]
            + self.matrix[3][0] * self.matrix[1][3] * self.matrix[2][2];

        inverted[2][0] = self.matrix[1][0] * self.matrix[2][1] * self.matrix[3][3]
            - self.matrix[1][0] * self.matrix[2][3] * self.matrix[3][1]
            - self.matrix[2][0] * self.matrix[1][1] * self.matrix[3][3]
            + self.matrix[2][0] * self.matrix[1][3] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[1][1] * self.matrix[2][3]
            - self.matrix[3][0] * self.matrix[1][3] * self.matrix[2][1];

        inverted[3][0] = -self.matrix[1][0] * self.matrix[2][1] * self.matrix[3][2]
            + self.matrix[1][0] * self.matrix[2][2] * self.matrix[3][1]
            + self.matrix[2][0] * self.matrix[1][1] * self.matrix[3][2]
            - self.matrix[2][0] * self.matrix[1][2] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[1][1] * self.matrix[2][2]
            + self.matrix[3][0] * self.matrix[1][2] * self.matrix[2][1];

        inverted[0][1] = -self.matrix[0][1] * self.matrix[2][2] * self.matrix[3][3]
            + self.matrix[0][1] * self.matrix[2][3] * self.matrix[3][2]
            + self.matrix[2][1] * self.matrix[0][2] * self.matrix[3][3]
            - self.matrix[2][1] * self.matrix[0][3] * self.matrix[3][2]
            - self.matrix[3][1] * self.matrix[0][2] * self.matrix[2][3]
            + self.matrix[3][1] * self.matrix[0][3] * self.matrix[2][2];

        inverted[1][1] = self.matrix[0][0] * self.matrix[2][2] * self.matrix[3][3]
            - self.matrix[0][0] * self.matrix[2][3] * self.matrix[3][2]
            - self.matrix[2][0] * self.matrix[0][2] * self.matrix[3][3]
            + self.matrix[2][0] * self.matrix[0][3] * self.matrix[3][2]
            + self.matrix[3][0] * self.matrix[0][2] * self.matrix[2][3]
            - self.matrix[3][0] * self.matrix[0][3] * self.matrix[2][2];

        inverted[2][1] = -self.matrix[0][0] * self.matrix[2][1] * self.matrix[3][3]
            + self.matrix[0][0] * self.matrix[2][3] * self.matrix[3][1]
            + self.matrix[2][0] * self.matrix[0][1] * self.matrix[3][3]
            - self.matrix[2][0] * self.matrix[0][3] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[0][1] * self.matrix[2][3]
            + self.matrix[3][0] * self.matrix[0][3] * self.matrix[2][1];

        inverted[3][1] = self.matrix[0][0] * self.matrix[2][1] * self.matrix[3][2]
            - self.matrix[0][0] * self.matrix[2][2] * self.matrix[3][1]
            - self.matrix[2][0] * self.matrix[0][1] * self.matrix[3][2]
            + self.matrix[2][0] * self.matrix[0][2] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[0][1] * self.matrix[2][2]
            - self.matrix[3][0] * self.matrix[0][2] * self.matrix[2][1];

        inverted[0][2] = self.matrix[0][1] * self.matrix[1][2] * self.matrix[3][3]
            - self.matrix[0][1] * self.matrix[1][3] * self.matrix[3][2]
            - self.matrix[1][1] * self.matrix[0][2] * self.matrix[3][3]
            + self.matrix[1][1] * self.matrix[0][3] * self.matrix[3][2]
            + self.matrix[3][1] * self.matrix[0][2] * self.matrix[1][3]
            - self.matrix[3][1] * self.matrix[0][3] * self.matrix[1][2];

        inverted[1][2] = -self.matrix[0][0] * self.matrix[1][2] * self.matrix[3][3]
            + self.matrix[0][0] * self.matrix[1][3] * self.matrix[3][2]
            + self.matrix[1][0] * self.matrix[0][2] * self.matrix[3][3]
            - self.matrix[1][0] * self.matrix[0][3] * self.matrix[3][2]
            - self.matrix[3][0] * self.matrix[0][2] * self.matrix[1][3]
            + self.matrix[3][0] * self.matrix[0][3] * self.matrix[1][2];

        inverted[2][2] = self.matrix[0][0] * self.matrix[1][1] * self.matrix[3][3]
            - self.matrix[0][0] * self.matrix[1][3] * self.matrix[3][1]
            - self.matrix[1][0] * self.matrix[0][1] * self.matrix[3][3]
            + self.matrix[1][0] * self.matrix[0][3] * self.matrix[3][1]
            + self.matrix[3][0] * self.matrix[0][1] * self.matrix[1][3]
            - self.matrix[3][0] * self.matrix[0][3] * self.matrix[1][1];

        inverted[3][2] = -self.matrix[0][0] * self.matrix[1][1] * self.matrix[3][2]
            + self.matrix[0][0] * self.matrix[1][2] * self.matrix[3][1]
            + self.matrix[1][0] * self.matrix[0][1] * self.matrix[3][2]
            - self.matrix[1][0] * self.matrix[0][2] * self.matrix[3][1]
            - self.matrix[3][0] * self.matrix[0][1] * self.matrix[1][2]
            + self.matrix[3][0] * self.matrix[0][2] * self.matrix[1][1];

        inverted[0][3] = -self.matrix[0][1] * self.matrix[1][2] * self.matrix[2][3]
            + self.matrix[0][1] * self.matrix[1][3] * self.matrix[2][2]
            + self.matrix[1][1] * self.matrix[0][2] * self.matrix[2][3]
            - self.matrix[1][1] * self.matrix[0][3] * self.matrix[2][2]
            - self.matrix[2][1] * self.matrix[0][2] * self.matrix[1][3]
            + self.matrix[2][1] * self.matrix[0][3] * self.matrix[1][2];

        inverted[1][3] = self.matrix[0][0] * self.matrix[1][2] * self.matrix[2][3]
            - self.matrix[0][0] * self.matrix[1][3] * self.matrix[2][2]
            - self.matrix[1][0] * self.matrix[0][2] * self.matrix[2][3]
            + self.matrix[1][0] * self.matrix[0][3] * self.matrix[2][2]
            + self.matrix[2][0] * self.matrix[0][2] * self.matrix[1][3]
            - self.matrix[2][0] * self.matrix[0][3] * self.matrix[1][2];

        inverted[2][3] = -self.matrix[0][0] * self.matrix[1][1] * self.matrix[2][3]
            + self.matrix[0][0] * self.matrix[1][3] * self.matrix[2][1]
            + self.matrix[1][0] * self.matrix[0][1] * self.matrix[2][3]
            - self.matrix[1][0] * self.matrix[0][3] * self.matrix[2][1]
            - self.matrix[2][0] * self.matrix[0][1] * self.matrix[1][3]
            + self.matrix[2][0] * self.matrix[0][3] * self.matrix[1][1];

        inverted[3][3] = self.matrix[0][0] * self.matrix[1][1] * self.matrix[2][2]
            - self.matrix[0][0] * self.matrix[1][2] * self.matrix[2][1]
            - self.matrix[1][0] * self.matrix[0][1] * self.matrix[2][2]
            + self.matrix[1][0] * self.matrix[0][2] * self.matrix[2][1]
            + self.matrix[2][0] * self.matrix[0][1] * self.matrix[1][2]
            - self.matrix[2][0] * self.matrix[0][2] * self.matrix[1][1];

        let det = self.matrix[0][0] * inverted[0][0]
            + self.matrix[0][1] * inverted[1][0]
            + self.matrix[0][2] * inverted[2][0]
            + self.matrix[0][3] * inverted[3][0];
        if det == 0.0 {
            panic!("Matrix is not invertible");
        }

        let det = 1.0 / det;
        for row in inverted.iter_mut() {
            for element in row.iter_mut() {
                *element *= det;
            }
        }

        Self { matrix: inverted }
    }

    pub fn transposed(&self) -> Self {
        let mut transposed = self.matrix;

        for (x, row) in transposed.iter_mut().enumerate() {
            for (y, element) in row.iter_mut().enumerate() {
                *element = self.matrix[y][x];
            }
        }

        Self::from_matrix(transposed)
    }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result: [[f32; 4]; 4] = [[0.0; 4]; 4];

        for (x, row) in result.iter_mut().enumerate() {
            for (y, element) in row.iter_mut().enumerate() {
                *element = self.matrix[x][0] * rhs.matrix[0][y]
                    + self.matrix[x][1] * rhs.matrix[1][y]
                    + self.matrix[x][2] * rhs.matrix[2][y]
                    + self.matrix[x][3] * rhs.matrix[3][y];
            }
        }
        
        Self { matrix: result }
    }
}
