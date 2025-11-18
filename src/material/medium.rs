use crate::{
    math::{Color, EPS, PI, Vec3, cross, dot, fmax},
    random::XorRand,
};

pub struct Medium {
    pub coeff_sc: Color,
    pub coeff_ex: Color,
    pub g: f32,
}

#[allow(unused)]
impl Medium {
    pub fn new(coeff_sc: Color, coeff_ab: Color, g: f32) -> Self {
        let coeff_sc = Vec3(
            fmax(coeff_sc.0, 1e-4),
            fmax(coeff_sc.1, 1e-4),
            fmax(coeff_sc.2, 1e-4),
        );

        Medium {
            coeff_sc,
            coeff_ex: coeff_sc + coeff_ab,
            g,
        }
    }
}

pub fn pdfs_sample_distance(coeff_ex: &Vec3, distance: f64) -> Vec3 {
    let calc = |c: f64| (-distance * c).exp();
    Vec3(calc(coeff_ex.0), calc(coeff_ex.1), calc(coeff_ex.2))
}

pub fn sample_phase(wo: &Vec3, g: f64, rand: &mut XorRand) -> (Vec3, f64) {
    let w = *wo;
    let u = if w.0.abs() > EPS {
        cross(Vec3(0., 1., 0.), w).normalize()
    } else {
        cross(Vec3(1., 0., 0.), w).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();

    let cos_theta = if g.abs() > EPS {
        let r = (1. - g * g) / (1. + g - 2. * g * rand.next01());
        -1. / (2. * g) * (1. + g * g - r * r)
    } else {
        1. - 2. * rand.next01()
    };
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();

    (
        u * sin_theta * phi.cos() + v * sin_theta * phi.sin() + w * cos_theta,
        1. / (4. * PI) * (1. - g * g) / (1. + g * g + 2. * g * cos_theta).powf(1.5),
    )
}

pub fn pdf_phase(wi: &Vec3, g: f64, wo: &Vec3) -> f64 {
    let dot = dot(*wo, *wi);
    1. / (4. * PI) * (1. - g * g) / (1. + g * g + 2. * g * dot).powf(1.5)
}

pub fn sample_rgb(pdf: &Vec3, coeff_ex: &Vec3, rand: &mut XorRand) -> f64 {
    let cdf = Vec3(pdf.0, 1. - pdf.2, 1.);
    let r = rand.next01();
    if r < cdf.0 {
        coeff_ex.0
    } else if r >= cdf.1 {
        coeff_ex.2
    } else {
        coeff_ex.1
    }
}
