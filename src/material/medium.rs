use crate::{
    math::{EPS, PI, Vec3, cross, dot, fmax},
    random::XorRand,
};

#[derive(PartialEq, Clone, Copy)]
pub struct Medium {
    pub coeff_sc: f64,
    pub coeff_ex: f64,
}

#[allow(unused)]
impl Medium {
    pub fn new(coeff_sc: f64, coeff_ab: f64) -> Self {
        Medium {
            coeff_sc: fmax(coeff_sc, 0.01),
            coeff_ex: coeff_sc + coeff_ab,
        }
    }
}

pub fn sample_phase(dir: &Vec3, rand: &mut XorRand) -> (Vec3, f64) {
    let w = *dir;
    let u = if w.0.abs() > EPS {
        cross(Vec3(0., 1., 0.), w).normalize()
    } else {
        cross(Vec3(1., 0., 0.), w).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();

    let g = 0.8;
    let cos_theta = {
        let r = (1. - g * g) / (1. + g - 2. * g * rand.next01());
        -1. / (2. * g) * (1. + g * g - r * r)
    };
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();

    (
        u * sin_theta * phi.cos() + v * sin_theta * phi.sin() + w * cos_theta,
        1. / (4. * PI) * (1. - g * g) / (1. + g * g + 2. * g * cos_theta).powf(1.5),
    )
}

pub fn pdf_phase(dir: &Vec3, prev_dir: &Vec3) -> f64 {
    let dot = dot(*prev_dir, *dir);
    let g = 0.8;
    1. / (4. * PI) * (1. - g * g) / (1. + g * g + 2. * g * dot).powf(1.5)
}
