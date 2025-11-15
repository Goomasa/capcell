use crate::math::{Color, Vec3, fmax, lerp_vec3};

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Bxdf {
    NoSurface,
    Lambertian,
    Light,
    IdealMirror,
    IdealGlass {
        ior: f64,
    },
    MicroBrdf {
        ax: f64,
        ay: f64,
    },
    MicroBtdf {
        a: f64,
        ior: f64,
    },
    CompositeBrdf {
        basecolor: Color,
        metalic: f64,
        highlight: Color,
        roughness: f64,
    },
}

#[allow(unused)]
impl Bxdf {
    pub fn is_light(&self) -> bool {
        match self {
            Self::Light => true,
            _ => false,
        }
    }

    pub fn set_comp(basecolor: Color, metalic: f64, specular: f64, roughness: f64) -> Self {
        let highlight = lerp_vec3(0.08 * specular, &basecolor, &Vec3::new(metalic));
        Bxdf::CompositeBrdf {
            basecolor,
            metalic,
            highlight,
            roughness: fmax(roughness, 0.01),
        }
    }
}
