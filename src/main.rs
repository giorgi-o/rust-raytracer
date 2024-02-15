#![allow(dead_code)]

use std::{path::PathBuf, process::Command, time::Instant};

use environments::environment::Environment;

use scene_file::{ParseError, SceneFile};

use crate::cameras::{camera::Camera, full_camera::FullCamera};

mod core {
    pub mod colour;
    pub mod framebuffer;
    pub mod hit;
    pub mod photon;
    pub mod photon_tree;
    pub mod ray;
    pub mod tex_coords;
    pub mod transform;
    pub mod vector;
    pub mod vertex;
}

mod environments {
    pub mod environment;
    pub mod photon_scene;
    pub mod scene;
}

mod cameras {
    pub mod camera;
    pub mod full_camera;
}

mod materials {
    pub mod compound_material;
    pub mod falsecolour_material;
    pub mod global_material;
    pub mod material;
    pub mod phong_material;
    pub mod texture;
}

mod lights {
    pub mod directional_light;
    pub mod directional_point_light;
    pub mod light;
    pub mod point_light;
}

mod objects {
    pub mod csg_object;
    pub mod cuboid_object;
    pub mod object;
    pub mod plane_object;
    pub mod polymesh_object;
    pub mod quadratic_object;
    pub mod sphere_object;
    pub mod triangle_object;
}

mod scene_file;

fn parse_path(path: &str) -> PathBuf {
    let mut parsed = std::env::current_dir().unwrap();
    for part in path.split('/') {
        parsed.push(part);
    }
    parsed
}

fn main() {
    // get txt filename from first argv
    let scene_filename = std::env::args()
        .nth(1)
        .unwrap_or("assets/scenes/scene2.txt".to_string());

    // when assets/scene.txt changes, re-render
    let get_last_modified = || {
        std::fs::metadata(&scene_filename)
            .expect("Failed to get metadata for scene file")
            .modified()
            .expect("Failed to get modified time for scene file")
    };

    loop {
        let last_modified = get_last_modified();

        render(&scene_filename);
        println!("Waiting for changes to {scene_filename}...");

        loop {
            if get_last_modified() > last_modified {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

fn build_scene(
    scene_filename: &str,
) -> Result<(Box<dyn Environment>, Box<FullCamera>), ParseError> {
    SceneFile::from_path(&parse_path(scene_filename))
}

fn render(scene_filename: &str) {
    let start = Instant::now();

    let (mut scene, camera) = match build_scene(scene_filename) {
        Ok(scene) => scene,
        Err(e) => {
            println!("Failed to build scene! {:?}", e);
            return;
        }
    };
    let build_scene_end = Instant::now();

    let framebuffer = camera.render(scene.as_mut());
    let render_end = Instant::now();

    let rgb_outpath = parse_path("render/rgb.ppm");
    framebuffer.write_rgb_file(&rgb_outpath);
    framebuffer.write_depth_file(&parse_path("render/depth.ppm"));
    let write_end = Instant::now();

    println!("Running FFmpeg...");
    ffmpeg_ppm_to_png(rgb_outpath);
    let ffmpeg_end = Instant::now();

    println!(
        "Done! Took {:.2} seconds - build scene: {:.2}, render: {:.2}, write: {:.2}, ffmpeg: {:.2}",
        (ffmpeg_end - start).as_secs_f32(),
        (build_scene_end - start).as_secs_f32(),
        (render_end - build_scene_end).as_secs_f32(),
        (write_end - render_end).as_secs_f32(),
        (ffmpeg_end - write_end).as_secs_f32()
    );
}

fn ffmpeg_ppm_to_png(ppm_filename: PathBuf) {
    let png_filename = ppm_filename.with_extension("png");
    Command::new("ffmpeg")
        .arg("-y")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("warning")
        .arg("-i")
        .arg(ppm_filename)
        .arg(png_filename)
        .output()
        .expect("Failed to execute ffmpeg");
}
