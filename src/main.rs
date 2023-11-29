#![allow(dead_code)]

use std::sync::Arc;
use std::{path::PathBuf, process::Command};

use core::scene::Scene;
use materials::compound_material::CompoundMaterial;
use materials::global_material::GlobalMaterial;
use materials::phong_material::Phong;
use objects::cuboid_object::Cuboid;
use objects::plane_object::Plane;

use crate::objects::sphere_object::Sphere;
use crate::{
    cameras::{camera::Camera, full_camera::FullCamera},
    core::{colour::Colour, vector::Vector, vertex::Vertex},
    lights::directional_light::DirectionalLight,
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
}

mod lights {
    pub mod directional_light;
    pub mod light;
}

mod objects {
    pub mod cuboid_object;
    pub mod object;
    pub mod plane_object;
    pub mod polymesh_object;
    pub mod sphere_object;
    pub mod triangle_object;
    pub mod csg_object;
    pub mod quadratic_object;
}

mod scene_file;

fn parse_path(path: &str) -> PathBuf {
    let mut parsed = std::env::current_dir().unwrap();
    for part in path.split('/') {
        parsed.push(part);
    }
    parsed
}

fn build_scene() -> Scene {
    let mut scene = Scene::new();

    let floor = Plane::new_from_point(
        &Vertex::new_xyz(0.0, 0.0, 0.0),
        &Vector::new(0.0, 1.0, 0.0),
        CompoundMaterial::new_simple(Colour::white(), 0.0),
    );
    scene.add_object(floor);

    let back_wall = Plane::new_from_point(
        &Vertex::new_xyz(0.0, 0.0, 15.0),
        &Vector::new(0.0, 0.0, -1.0),
        CompoundMaterial::new_simple(Colour::new(0.5, 0.0, 0.0), 0.0),
    );
    scene.add_object(back_wall);

    let left_wall = Plane::new_from_point(
        &Vertex::new_xyz(-5.0, 0.0, 0.0),
        &Vector::new(1.0, 0.0, 0.0),
        CompoundMaterial::new_simple(Colour::new(0.0, 0.5, 0.0), 0.0),
    );
    scene.add_object(left_wall);

    let right_wall = Plane::new_from_point(
        &Vertex::new_xyz(10.0, 0.0, 0.0),
        &Vector::new(-1.0, 0.0, 0.0),
        CompoundMaterial::new_simple(Colour::new(0.0, 0.0, 0.5), 0.0),
    );
    // scene.add_object(right_wall);

    // LEFT

    let sphere1 = Sphere::new(
        Vertex::new_xyz(-2.0, 1.5, 5.0),
        1.0,
        CompoundMaterial::new_simple(Colour::new(1.0, 0.5, 0.0), 0.3),
    );
    scene.add_object(sphere1);

    let cube = Cuboid::new(
        Vertex::new_xyz(-1.3, 0.0, 3.0),
        Vector::new(0.7, 0.7, 0.7),
        // Arc::new(GlobalMaterial::new(
        //     Colour::black(),
        //     Colour::white(),
        //     1.0,
        // )),
        CompoundMaterial::new_transparent(Colour::new(1.0, 1.0, 0.0), 0.9, 1.2),
    );
    scene.add_object(cube);

    // RIGHT

    let sphere2 = Sphere::new(
        Vertex::new_xyz(1.0, 1.0, 4.0),
        1.0,
        // CompoundMaterial::new_simple(Colour::new(0.0, 1.0, 1.0), 1.0),
        // CompoundMaterial::new_transparent(Colour::new(0.0, 1.0, 1.0), 0.9, 1.2),
        GlobalMaterial::new(Colour::new(1.0, 1.0, 1.0), Colour::white(), 2.4),
    );
    scene.add_object(sphere2);

    let cube2 = Cuboid::new(
        Vertex::new_xyz(-0.5, 0.0, 6.0),
        Vector::new(2.0, 1.0, 1.0),
        Phong::new(
            Colour::black(),
            Colour::new(0.0, 0.5, 0.5),
            Colour::black(),
            100.0,
        ),
    );
    scene.add_object(cube2);

    let sun_direction = Vector::new(-1.0, -0.9, 1.0);
    let sun_colour = Colour::white();
    let sun = DirectionalLight::new(sun_direction, sun_colour);
    scene.add_light(sun);

    scene
}

fn main() {
    println!("Hello, world!");

    let width = 1024;
    let height = 1024;

    let scene = build_scene();

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
