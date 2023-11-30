#![allow(dead_code)]

use std::{path::PathBuf, process::Command};

use core::scene::Scene;

use scene_file::{ParseError, SceneFile};

use crate::{
    cameras::{camera::Camera, full_camera::FullCamera},
    core::{vector::Vector, vertex::Vertex},
};

mod core {
    pub mod colour;
    pub mod environment;
    pub mod framebuffer;
    pub mod hit;
    pub mod ray;
    pub mod scene;
    pub mod tex_coords;
    pub mod transform;
    pub mod vector;
    pub mod vertex;
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
    pub mod texture_material;
}

mod lights {
    pub mod directional_light;
    pub mod light;
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
    // when assets/scene.txt changes, re-render
    let get_last_modified = || {
        std::fs::metadata("assets/scene.txt")
            .expect("Failed to get metadata for assets/scene.txt")
            .modified()
            .expect("Failed to get modified time for assets/scene.txt")
    };

    loop {
        render();
        println!("Waiting for changes to assets/scene.txt...");

        let last_modified = get_last_modified();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));

            if get_last_modified() > last_modified {
                break;
            }
        }
    }
}

fn build_scene() -> Result<Scene, ParseError> {
    SceneFile::from_path(&parse_path("assets/scene.txt"))
}

fn render() {
    let width = 1024;
    let height = 1024;

    let scene = match build_scene() {
        Ok(scene) => scene,
        Err(e) => {
            println!("Failed to build scene! {:?}", e);
            return;
        }
    };

    // "default" camera position
    let position = Vertex::new_xyz(0.0, 3.0, 0.0);
    let lookat = Vector::new(0.0, 0.5, 1.0).normalised();
    let up = Vector::new(0.0, lookat.z, -lookat.y);
    let fov = 40f32.to_radians();

    let camera = FullCamera::new(width, height, fov, position, lookat, up);

    let framebuffer = camera.render(&scene);

    let rgb_outpath = parse_path("render/rgb.ppm");
    framebuffer.write_rgb_file(&rgb_outpath);
    framebuffer.write_depth_file(&parse_path("render/depth.ppm"));

    println!("Running FFmpeg...");
    ffmpeg_ppm_to_png(rgb_outpath);

    println!("Done!");
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
