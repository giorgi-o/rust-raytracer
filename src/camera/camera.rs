use crate::core::{environment::Environment, framebuffer::Framebuffer};

pub trait Camera {
    fn render<E: Environment>(&self, environment: E) -> Framebuffer {
        const NUM_THREADS: u32 = 4;
    }

    fn render_rows<E: Environment>(&self, environment: E, start_y: u32, end_y: u32) -> Framebuffer;
}