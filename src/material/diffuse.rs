use crate::{
    math::{EPS, PI, PI_INV, Vec3, cross, dot, fmax},
    random::XorRand,
};

pub fn sample_cos_hemisphere(normal: &Vec3, rand: &mut XorRand) -> Vec3 {
    let w = *normal;
    let u = if w.0.abs() > EPS {
        cross(w, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(w, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();
    let sin_theta_sq = rand.next01();
    let sin_theta = sin_theta_sq.sqrt();

    (u * sin_theta * (phi.cos()) + v * sin_theta * (phi.sin()) + w * ((1. - sin_theta_sq).sqrt()))
        .normalize()
}

pub fn pdf_cos_hemisphere(dir: &Vec3, normal: &Vec3) -> f64 {
    fmax(dot(*dir, *normal).abs() * PI_INV, 0.)
}
