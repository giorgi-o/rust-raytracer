use std::io::Write;

use crate::{
    core::{
        framebuffer::FrameBuffer, ray::Ray, transform::Transform, vector::Vector, vertex::Vertex,
    },
    environments::environment::{Environment, RaytraceResult},
};

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
        let right_angle = std::f32::consts::FRAC_PI_2;
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

impl Camera for FullCamera {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn render_rows<E: Environment>(
        &self,
        environment: &E,
        start_y: u32,
        end_y: u32,
    ) -> FrameBuffer {
        let mut framebuffer = FrameBuffer::new(self.width, end_y - start_y);
        let start = std::time::Instant::now();

        let is_first_thread = start_y == 0;
        let mut stdout_lock = is_first_thread.then(|| std::io::stdout().lock());

        for y in start_y..end_y {
            for x in 0..self.width {
                let ray = self.get_ray_pixel(x, y);
                let RaytraceResult { colour, depth } = environment.raytrace(&ray);

                framebuffer.plot_pixel(x, y - start_y, &colour);
                framebuffer.plot_depth(x, y - start_y, depth);
            }

            // print ETA

            if !is_first_thread {
                continue;
            }

            // only print for first, last two, and every 5th row
            if y > 0 && y < end_y - 1 && y % 5 != 0 {
                continue;
            }

            let Some(stdout) = &mut stdout_lock else {
                panic!("stdout lock is None");
            };

            let height = end_y;
            let progress = (y + 1) as f32 / height as f32;

            let elapsed = start.elapsed().as_secs_f32();
            let eta = elapsed / progress - elapsed;
            let percent = (progress * 100.0) as u32;

            // print!("\r{percent}% {elapsed:.2}s elapsed, {eta:.2}s ETA");
            let _ = write!(stdout, "\r{percent}% {elapsed:.2}s elapsed, {eta:.2}s ETA");
            let _ = stdout.flush();
        }

        if start_y == 0 {
            println!();
        }

        framebuffer
    }
}
