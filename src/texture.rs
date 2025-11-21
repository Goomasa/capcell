use std::fs::File;

use crate::math::{Color, PI, Vec3};

#[allow(unused)]
pub enum Texture<'a> {
    Solid(Color),
    Checker {
        div: u32,
        col1: Color,
        col2: Color,
    },
    Image {
        data: &'a Vec<Color>,
        width: usize,
        height: usize,
    },
    Gradation,
}

#[allow(unused)]
impl<'a> Texture<'a> {
    pub fn set_solid(color: Color) -> Self {
        Texture::Solid(color)
    }

    pub fn set_checker(div: u32, col1: Color, col2: Color) -> Self {
        Texture::Checker { div, col1, col2 }
    }

    pub fn set_image(data: &'a Vec<Color>, width: usize, height: usize) -> Self {
        Texture::Image {
            data,
            width,
            height,
        }
    }

    pub fn get_color(&self, u: f64, v: f64) -> Color {
        match *self {
            Texture::Solid(color) => color,
            Texture::Checker { div, col1, col2 } => {
                let id_u = u * (div as f64);
                let id_v = v * (div as f64);
                if (id_u as u32 + id_v as u32) % 2 == 0 {
                    col1
                } else {
                    col2
                }
            }
            Texture::Image {
                data,
                width,
                height,
            } => {
                let id_u = (width as f64 * u) as usize;
                let id_v = (height as f64 * v) as usize;
                let id = id_v * width + id_u;
                data[id]
            }
            Texture::Gradation => {
                let theta = PI * v;
                let phi = 2. * PI * u;
                (Vec3(
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ) + Vec3::new(1.))
                    / 2.
            }
        }
    }
}

#[allow(unused)]
pub fn load_hdr(path: &str) -> (Vec<Color>, usize, usize) {
    // return (pixel_data, width, height)
    let file = File::open(path).expect("failed to open hdr");
    let image = hdrldr::load(file).expect("failed to load hdr");

    let mut data = Vec::new();
    for rgb in image.data.iter() {
        data.push(Vec3(
            rgb.r.powf(2.2).clamp(0., 10.) as f64,
            rgb.g.powf(2.2).clamp(0., 10.) as f64,
            rgb.b.powf(2.2).clamp(0., 10.) as f64,
        ));
    }

    (data, image.width, image.height)
}
