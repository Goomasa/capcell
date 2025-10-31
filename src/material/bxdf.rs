use crate::math::Color;

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Bxdf {
    Lambertian,
    Light,
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
}
