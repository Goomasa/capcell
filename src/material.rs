use crate::{
    math::{Color, EPS, PI, PI_INV, Vec3, cross, dot, fmax},
    random::XorRand,
};

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Bxdf {
    Lambertian,
    Light,
    MicroBrdf { ax: f64, ay: f64 },
    MicroBtdf { a: f64, ior: f64 },
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

pub fn reflection_dir(normal: &Vec3, in_dir: &Vec3) -> Vec3 {
    *in_dir + *normal * dot(*in_dir, *normal) * (-2.)
}

pub fn refraction_dir(
    ior_from: f64,
    ior_to: f64,
    normal: &Vec3,
    in_dir: &Vec3,
    rand: &mut XorRand,
) -> (bool, Vec3, f64) {
    //return (is_refract, new_dir, reflectance)
    let reflection_dir = reflection_dir(normal, in_dir);
    let nnt = ior_to / ior_from;
    let ddn = dot(*in_dir, *normal);
    let cos2t = 1. - nnt * nnt * (1. - ddn * ddn);

    if cos2t < 0. {
        return (false, reflection_dir, 1.0);
    }

    let refraction_dir = (-*normal * (cos2t.sqrt()) + (*in_dir - *normal * ddn) * nnt).normalize();
    let a = ior_from - ior_to;
    let b = ior_from + ior_to;
    let r0 = (a * a) / (b * b);
    let c = if ior_from > ior_to {
        1. + ddn
    } else {
        1. - dot(refraction_dir, -*normal)
    };
    let fresnel_reflectance = r0 + (1. - r0) * c.powi(5);

    if rand.next01() < fresnel_reflectance {
        (false, reflection_dir, fresnel_reflectance)
    } else {
        (true, refraction_dir, fresnel_reflectance)
    }
}

pub fn sample_ggx_vndf(normal: &Vec3, wo: &Vec3, ax: f64, ay: f64, rand: &mut XorRand) -> Vec3 {
    let u = if normal.0.abs() > EPS {
        cross(*normal, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(*normal, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(*normal, u);

    let ve = Vec3(dot(u, *wo), dot(v, *wo), dot(*normal, *wo));

    let vh = Vec3(ax * ve.0, ay * ve.1, ve.2).normalize();
    let lensq = vh.0 * vh.0 + vh.1 * vh.1;
    let t1 = if lensq > 0. {
        Vec3(-vh.1, vh.0, 0.) / lensq.sqrt()
    } else {
        Vec3(1., 0., 0.)
    };
    let t2 = cross(vh, t1);

    let r = rand.next01().sqrt();
    let phi = 2. * PI * rand.next01();
    let p1 = r * phi.cos();
    let mut p2 = r * phi.sin();
    let s = 0.5 * (1. + vh.2);
    p2 = (1. - s) * (1. - p1 * p1).sqrt() + s * p2;

    let nh = t1 * p1 + t2 * p2 + vh * fmax(1. - p1 * p1 - p2 * p2, 0.).sqrt();
    let vn = Vec3(ax * nh.0, ay * nh.1, fmax(nh.2, 0.));
    (u * vn.0 + v * vn.1 + *normal * vn.2).normalize()
}

pub fn shadow_mask_fn(ax: f64, ay: f64, w: &Vec3, normal: &Vec3) -> f64 {
    let alpha2 = ggx_alpha2(ax, ay, w, normal);
    let cos_theta = dot(*w, *normal);
    let tan_theta2 = 1. / (cos_theta * cos_theta) - 1.;

    2. / (1. + (1. + alpha2 * tan_theta2).sqrt())
}

fn ggx_alpha2(ax: f64, ay: f64, w: &Vec3, normal: &Vec3) -> f64 {
    if ax == ay {
        return ax * ax;
    }

    let wo_dash = *w - *normal * dot(*w, *normal);
    let u = if normal.0.abs() > EPS {
        cross(*normal, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(*normal, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(*normal, u);

    let tan_phi = dot(wo_dash, v) / dot(wo_dash, u);
    let cos_phi2 = 1. / (1. + tan_phi * tan_phi);
    ax * ax * cos_phi2 + ay * ay * (1. - cos_phi2)
}

pub fn fresnel_color(f0: &Color, wi: &Vec3, vn: &Vec3) -> Color {
    *f0 + (Vec3::new(1.) - *f0) * (1. - dot(*wi, *vn)).clamp(0., 1.).powf(5.)
}

pub fn fresnel_ior(ior_from: f64, ior_to: f64, wo: &Vec3, vn: &Vec3) -> f64 {
    let a = ior_from - ior_to;
    let b = ior_from + ior_to;
    let r0 = (a * a) / (b * b);
    let c = if ior_from > ior_to {
        1. + dot(*wo, -*vn)
    } else {
        1. - dot(*wo, -*vn)
    };

    r0 + (1. - r0) * c.powf(5.)
}

pub fn ggx_normal_df(ax: f64, ay: f64, normal: &Vec3, wm: &Vec3) -> f64 {
    let cos_theta = dot(*wm, *normal);
    let tan_theta2 = 1. / (cos_theta * cos_theta) - 1.;

    let vn_dash = *wm - *normal * cos_theta;
    let u = if normal.0.abs() > EPS {
        cross(*normal, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(*normal, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(*normal, u);

    let tan_phi = dot(vn_dash, v) / dot(vn_dash, u);
    let cos_phi2 = 1. / (1. + tan_phi * tan_phi);

    if ax == 0. || ay == 0. {
        0.
    } else {
        let s = 1. + (cos_phi2 / (ax * ax) + (1. - cos_phi2) / (ay * ay)) * tan_theta2;
        PI_INV / (ax * ay * cos_theta.powf(4.) * s * s)
    }
}

pub fn micro_btdf_j(ior_from: f64, ior_to: f64, wo: &Vec3, wi: &Vec3, wm: &Vec3) -> f64 {
    let dot_wo_wm = dot(*wo, *wm);
    ior_to * ior_to * dot_wo_wm.abs() / (ior_from * dot(*wi, *wm) + ior_to * dot_wo_wm).powf(2.)
}
