use std::{
    fs::File,
    io::{BufWriter, Write},
};

use super::colour::Colour;

#[derive(Clone)]
struct Pixel {
    pub colour: Colour,
    pub depth: f32,
}

impl Pixel {
    pub fn new(red: f32, green: f32, blue: f32, depth: f32) -> Self {
        Self {
            colour: Colour::new_rgb(red, green, blue),
            depth,
        }
    }

    pub fn black() -> Self {
        Self {
            colour: Colour::black(),
            depth: 0.0,
        }
    }
}

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pixels: Vec<Pixel>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        if width >= 2048 || height >= 2048 {
            panic!("Invalid framebuffer size");
        }

        let mut framebuffer = Vec::new();
        framebuffer.resize((width * height) as usize, Pixel::black());

        Self {
            width,
            height,
            pixels: framebuffer,
        }
    }

    pub fn combine_rows(framebuffers: Vec<Self>) -> Self {
        let width = framebuffers[0].width;
        let height = framebuffers.iter().fold(0, |acc, fb| {
            assert!(fb.width == width);
            acc + fb.height
        });

        let mut pixels = Vec::with_capacity((width * height) as usize);
        for fb in framebuffers {
            pixels.extend(fb.pixels);
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    fn framebuffer_index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width && y < self.height);
        (y * self.width + x) as usize
    }

    pub fn plot_pixel(&mut self, x: u32, y: u32, colour: &Colour) {
        let index = self.framebuffer_index(x, y);
        self.pixels[index].colour = *colour;
    }

    pub fn plot_depth(&mut self, x: u32, y: u32, depth: f32) {
        let index = self.framebuffer_index(x, y);
        self.pixels[index].depth = depth;
    }

    pub fn get_depth(&self, x: u32, y: u32) -> f32 {
        let index = self.framebuffer_index(x, y);
        self.pixels[index].depth
    }

    pub fn get_colour(&self, x: u32, y: u32) -> Colour {
        let index = self.framebuffer_index(x, y);
        self.pixels[index].colour
    }

    pub fn write_rgb_file(&self, filename: String) {
        assert!(filename.ends_with(".ppm"));

        let outfile = File::create(filename).unwrap();
        let mut writer = BufWriter::new(outfile);

        let header = format!("P6\n{} {}\n255\n", self.width, self.height);
        writer.write_all(header.as_bytes()).unwrap();

        for pixel in &self.pixels {
            // assume all colour values are between 0.0 and 1.0
            let red = (pixel.colour.r * 255.0) as u8;
            let green = (pixel.colour.g * 255.0) as u8;
            let blue = (pixel.colour.b * 255.0) as u8;

            writer.write_all(&[red, green, blue]).unwrap();
        }

        writer.flush().unwrap();
    }

    pub fn write_depth_file(&self, filename: String) {
        assert!(filename.ends_with(".pgm"));

        let outfile = File::create(filename).unwrap();
        let mut writer = BufWriter::new(outfile);

        let header = format!("P5\n{} {}\n255\n", self.width, self.height);
        writer.write_all(header.as_bytes()).unwrap();

        for pixel in &self.pixels {
            // assume all depth values are between 0.0 and 1.0
            let depth = (pixel.depth * 255.0) as u8;

            writer.write_all(&[depth]).unwrap();
        }

        writer.flush().unwrap();
    }
}
