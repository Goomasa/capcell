use crate::math::Color;

pub enum Texture {
    Solid(Color),
    Checker { div: u32, col1: Color, col2: Color },
}

#[allow(unused)]
impl Texture {
    pub fn set_solid(color: Color) -> Self {
        Texture::Solid(color)
    }

    pub fn set_checker(div: u32, col1: Color, col2: Color) -> Self {
        Texture::Checker { div, col1, col2 }
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
        }
    }
}
