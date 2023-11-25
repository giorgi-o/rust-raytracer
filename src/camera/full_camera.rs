use crate::core::{ray::Ray, vector::Vector, vertex::Vertex, transform::Transform, environment::Environment, framebuffer::Framebuffer};

use super::camera::Camera;

pub struct FullCamera {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub position: Vertex,
    pub lookat: Vector,
    pub up: Vector,
    pub right: Vector,
}

impl FullCamera {
    pub fn new(
        width: u32,
        height: u32,
        fov: f32,
        position: Vertex,
        mut lookat: Vector,
        mut up: Vector,
    ) -> Self {
        // coordinates are specified in "higher y is positive" space,
        // but we work with them in "higher y is negative" space
        lookat.y = -lookat.y;
        up.y = -up.y;

        lookat.normalise();
        up.normalise();

        // check that angle between lookat and up is 90
        let angle = lookat.angle(&up);
        let right_angle = std::f32::consts::FRAC_2_PI;
        if (angle - right_angle).abs() > 0.0001 {
            panic!("FullCamera right and up are not perpendicular");
        }

        let mut right = lookat.cross(&up);
        right.normalise();

        Self {
            width,
            height,
            fov,
            position,
            lookat,
            up,
            right,
        }
    }

    // given a pixel coordinate, compute the corresponding ray
    pub fn get_ray_pixel(&self, x: u32, y: u32) -> Ray {
        assert!(x < self.width && y < self.height);

        let fx = (x as f32 + 0.5) / self.width as f32; // 0 <= fx < 1
        let fy = (y as f32 + 0.5) / self.height as f32; // 0 <= fy < 1

        let position = self.position.clone();
        let mut direction = Vector::new(fx - 0.5, fy - 0.5, self.fov);

        let rotation_matrix = [
            [self.right.x, self.right.y, self.right.z],
            [self.up.x, self.up.y, self.up.z],
            [self.lookat.x, self.lookat.y, self.lookat.z],
        ];
        let rotation_matrix = Transform::from_rotation_matrix(rotation_matrix);
        let rotation_matrix = rotation_matrix.transposed();
        direction.apply_transform(&rotation_matrix);
        direction.normalise();

        Ray::new(position, direction)
    }

    
}

// impl Camera for FullCamera {
//     fn render<E: Environment>(&self, environment: E) -> Framebuffer {
//         let mut framebuffer = Framebuffer::new(self.width, self.height);

//         for y in 0..self.height {
//             for x in 0..self.width {
//                 let ray = self.get_ray_pixel(x, y);
//                 let colour = environment.get_colour(&ray);
//                 framebuffer.plot_pixel(x, y, &colour);
//             }
//         }

//         framebuffer
//     }
// }
