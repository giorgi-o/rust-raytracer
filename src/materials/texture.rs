use std::{
    io::{BufRead, Read},
    path::PathBuf,
    process::Command,
    sync::Arc,
};

use crate::{
    core::{colour::Colour, hit::Hit, tex_coords::TexCoords, vector::Vector},
    environments::photon_scene::PhotonScene,
    parse_path,
};

use super::{material::PhotonMaterial, phong_material::Phong};

pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Colour>,
}

impl Image {
    pub fn from_image(path: PathBuf) -> Result<Self, String> {
        let ppm_path = path.with_extension("ppm");
        if !ppm_path.exists() {
            // convert to ppm
            Command::new("ffmpeg")
                .arg("-y")
                .arg("-hide_banner")
                .arg("-loglevel")
                .arg("warning")
                .arg("-i")
                .arg(path)
                .arg(ppm_path.clone())
                .output()
                .map_err(|e| e.to_string())?;
        }

        Self::from_ppm(ppm_path)
    }

    pub fn from_ppm(path: PathBuf) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut reader = std::io::BufReader::new(file);

        fn read_until_whitespace(reader: &mut impl BufRead) -> Result<String, String> {
            let mut result = String::new();
            loop {
                let mut buf = [0; 1];
                reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
                if buf[0] == b' ' || buf[0] == b'\n' || buf[0] == b'\r' || buf[0] == b'\t' {
                    break;
                }
                result.push(buf[0] as char);
            }
            Ok(result)
        }
        fn read_u32(reader: &mut impl BufRead) -> Result<u32, String> {
            read_until_whitespace(reader)?
                .parse::<u32>()
                .map_err(|e| e.to_string())
        }

        if read_until_whitespace(&mut reader)? != "P6" {
            return Err("Invalid PPM file: expected P6".to_string());
        }

        let width = read_u32(&mut reader)?;
        let height = read_u32(&mut reader)?;
        let max_value = read_u32(&mut reader)?;

        let bytes_per_sample = if max_value <= 255 { 1 } else { 2 };

        let mut pixels: Vec<Colour> = Vec::with_capacity((width * height) as usize);
        for _ in 0..(width * height) {
            let mut buf = [0; 6];
            let buf = &mut buf[0..(bytes_per_sample * 3)];

            reader.read_exact(buf).map_err(|e| e.to_string())?;

            let (r, g, b) = if bytes_per_sample == 1 {
                (buf[0] as u16, buf[1] as u16, buf[2] as u16)
            } else {
                (
                    (buf[0] as u16) << 8 | (buf[1] as u16),
                    (buf[2] as u16) << 8 | (buf[3] as u16),
                    (buf[4] as u16) << 8 | (buf[5] as u16),
                )
            };
            let (r, g, b) = (
                (r as f32) / (max_value as f32),
                (g as f32) / (max_value as f32),
                (b as f32) / (max_value as f32),
            );

            pixels.push(Colour::new(r, g, b));
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    fn get_xy(&self, x: u32, y: u32) -> Colour {
        // assert!(x < self.width && y < self.height);
        // let framebuffer_index = y * self.width + x;
        let framebuffer_index =
            (y.rem_euclid(self.height)) * self.width + (x.rem_euclid(self.width));
        self.pixels[framebuffer_index as usize]
    }

    fn get_uv(&self, u: f32, v: f32) -> Colour {
        let x = (u.rem_euclid(1.0) * (self.width - 1) as f32).floor() as u32;
        let y = (v.rem_euclid(1.0) * (self.height - 1) as f32).floor() as u32;
        self.get_xy(x, y)
    }

    fn get(&self, tex_coords: impl Into<TexCoords>) -> Colour {
        let tex_coords = tex_coords.into();
        self.get_uv(tex_coords.u, tex_coords.v)
    }
}

pub struct Texture {
    pub diffuse: Image,
    pub normal: Option<Image>,
    pub roughness: Option<Image>,
    scale: f32,
    ambient_strength: f32,
    shininess: f32,
}

impl Texture {
    pub fn import(name: String, scale: f32, ambient_strength: f32, shininess: f32) -> Arc<Self> {
        let folder = parse_path(&format!("assets/textures/{}", name));

        let diffuse = Image::from_image(folder.join("diffuse.jpg")).unwrap();
        let normal = Image::from_image(folder.join("normal.jpg")).ok();
        let roughness = Image::from_image(folder.join("roughness.jpg")).ok();

        Arc::new(Self {
            diffuse,
            normal,
            roughness,
            scale,
            ambient_strength,
            shininess,
        })
    }
}

impl Phong for Texture {
    fn colour_at_hit(&self, hit: &Hit) -> Colour {
        let tex_coords = hit
            .tex_coords
            .as_ref()
            .expect("No texture coordinates")
            .clone();
        self.diffuse.get(tex_coords * (1.0 / self.scale))
    }

    fn ambient_strength(&self) -> f32 {
        self.ambient_strength
    }

    fn shininess(&self) -> f32 {
        self.shininess
    }

    fn normal(&self, tex_coords: &TexCoords) -> Option<Vector> {
        let Some(normal) = self.normal.as_ref() else {
            return None;
        };
        let normal = normal.get(tex_coords.clone() * self.scale);
        let normal = Vector::new(normal.r, normal.g, normal.b);
        let normal: Vector = normal * 2.0 - Vector::new(1.0, 1.0, 1.0);
        let normal = normal.normalised();
        // dbg!(normal);
        Some(normal)
    }

    fn photon_mapped(&self) -> &dyn PhotonMaterial {
        self
    }
}

impl PhotonMaterial for Texture {
    fn compute_photon(&self, scene: &PhotonScene, hit: &Hit, ldir: &Vector) -> Colour {
        self.diffuse(hit, ldir)
    }
}
