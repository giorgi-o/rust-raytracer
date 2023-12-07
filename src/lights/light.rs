use crate::{
    core::{colour::Colour, photon::Photon, vector::Vector, vertex::Vertex, ray::Ray},
    environments::photon_scene::PhotonScene,
};

pub trait Light: Send + Sync {
    // Get the direction towards the light at the point on the surface
    // return none if the surface is behind and not illuminated
    fn get_direction(&self, surface: &Vertex) -> Option<Vector>;

    // Get the intensity of the light in the direction of the surface
    fn get_intensity(&self, surface: &Vertex) -> Option<Colour>;

    // You will need additional light methods to support Photon-mapping.

    fn photon_light(self: Box<Self>) -> Box<dyn PhotonLight> {
        panic!("Light does not support photon mapping");
    }
}

pub trait PhotonLight: Light {
    fn shoot_photons_mt(&self, scene: &PhotonScene, num_photons: u32) -> Vec<Vec<Photon>> {
        let num_threads = std::thread::available_parallelism().map_or(4, |n| n.get()) as u32;
        let photons_per_thread = num_photons / num_threads;
        let extra_photons = num_photons % num_threads;
        println!("Spawning {num_threads} threads to shoot {photons_per_thread} photons each... ({extra_photons} extra)");

        std::thread::scope(|scope| {
            let mut threads = Vec::new();

            for thread_index in 0..num_threads {
                let mut num_photons = photons_per_thread;
                if thread_index == num_threads - 1 {
                    num_photons += extra_photons;
                }

                let thread = scope.spawn(move || {
                    self.shoot_regular_photons(scene, num_photons, thread_index == 0)
                });
                threads.push(thread);
            }

            let mut photons = Vec::new();

            for (i, threads) in threads.into_iter().enumerate() {
                photons.push(threads.join().unwrap());
                print!(
                    "{i}/{num_threads} threads finished shooting photons\r",
                    i = i + 1
                )
            }
            println!();

            photons
        })
    }

    fn shoot_regular_photons<'a>(
        &'a self,
        scene: &'a PhotonScene,
        num_photons: u32,
        first_thread: bool,
    ) -> Vec<Photon>;

    fn shoot_caustic_photons<'a>(
        &'a self,
        scene: &'a PhotonScene,
        caustic_rays: &[Ray],
        num_photons: u32,
        first_thread: bool,
    ) -> Vec<Photon>;
}
