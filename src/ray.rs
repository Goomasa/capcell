use crate::{
    Bxdf,
    material::medium::Medium,
    math::{Color, INF, Point3, Vec3},
};

pub struct Ray {
    pub org: Point3,
    pub dir: Vec3,
}

#[derive(Clone, Copy)]
pub struct HitRecord {
    pub distance: f64,
    pub hitpoint: Point3,
    pub normal: Vec3,
    pub color: Color,
    pub bxdf: Bxdf,
    pub id: i32,
    pub medium: Medium,
}

impl HitRecord {
    pub fn new() -> Self {
        HitRecord {
            distance: INF,
            hitpoint: Vec3::zero(),
            normal: Vec3::zero(),
            color: Vec3::zero(),
            bxdf: Bxdf::Lambertian,
            id: -1,
            medium: Medium {
                coeff_sc: -1.,
                coeff_ex: -1.,
                g: 0.,
            },
        }
    }

    pub fn init_with_distance(d: f64) -> Self {
        HitRecord {
            distance: d,
            hitpoint: Vec3::zero(),
            normal: Vec3::zero(),
            color: Vec3::zero(),
            bxdf: Bxdf::Lambertian,
            id: -1,
            medium: Medium {
                coeff_sc: -1.,
                coeff_ex: -1.,
                g: 0.,
            },
        }
    }
}
