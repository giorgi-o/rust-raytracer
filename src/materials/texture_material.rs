use std::{
    io::{BufRead, Read},
    path::PathBuf,
};

use crate::{
    core::{colour::Colour, tex_coords::TexCoords},
    parse_path,
};

pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Colour>,
}

impl Image {
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
        assert!(x < self.width && y < self.height);
        let framebuffer_index = y * self.width + x;
        self.pixels[framebuffer_index as usize]
    }

    fn get_uv(&self, u: f32, v: f32) -> Colour {
        let x = (u * (self.width - 1) as f32).round() as u32;
        let y = (v * (self.height - 1) as f32).round() as u32;
        self.get_xy(x, y)
    }

    fn get(&self, tex_coords: impl Into<TexCoords>) -> Colour {
        let tex_coords = tex_coords.into();
        self.get_uv(tex_coords.u, tex_coords.v)
    }
}

pub struct Texture {
    pub diffuse: Image,
    pub normal: Image,
    pub roughness: Image,
    scale: f32,
}

impl Texture {
    pub fn import(name: String, scale: f32) -> Self {
        let folder = parse_path(&format!("assets/textures/{}", name));

        let diffuse = Image::from_ppm(folder.join("diffuse.ppm")).unwrap();
        let normal = Image::from_ppm(folder.join("normal.ppm")).unwrap();
        let roughness = Image::from_ppm(folder.join("roughness.ppm")).unwrap();

        Self {
            diffuse,
            normal,
            roughness,
            scale,
        }
    }
}
