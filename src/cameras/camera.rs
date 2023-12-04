use crate::{core::framebuffer::FrameBuffer, environments::environment::Environment};

pub trait Camera: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn render<E>(&self, environment: &E) -> FrameBuffer
    where
        for<'a> &'a Self: Send,
        E: Environment + Send,
    {
        let num_threads = std::thread::available_parallelism().map_or(4, |n| n.get()) as u32;
        // let num_threads = 1;
        let rows_per_thread = self.height() / num_threads;
        let extra_rows = self.height() % num_threads;
        println!("Spawning {num_threads} threads to render {rows_per_thread} rows each...");

        let mut framebuffers = Vec::new();

        std::thread::scope(|scope| {
            let mut threads = Vec::new();

            for thread_index in 0..num_threads {
                let start_y = thread_index * rows_per_thread;
                let mut end_y = (thread_index + 1) * rows_per_thread;

                if thread_index == num_threads - 1 {
                    end_y += extra_rows;
                }

                let thread = scope.spawn(move || self.render_rows(environment, start_y, end_y));
                threads.push(thread);
            }

            for thread in threads {
                framebuffers.push(thread.join().unwrap());
            }
        });

        FrameBuffer::combine_rows(framebuffers)
    }

    fn render_rows<E: Environment>(&self, environment: &E, start_y: u32, end_y: u32)
        -> FrameBuffer;
}
