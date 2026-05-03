use crate::{
    Bxdf,
    material::medium::Medium,
    math::{Color, INF, Point3, Vec3},
};

#[derive(Clone)]
pub struct Ray {
    pub org: Point3,
    pub dir: Vec3,
}

pub struct HitRecord<'a> {
    pub distance: f64,
    pub hitpoint: Point3,
    pub normal: Vec3,
    pub color: Color,
    pub bxdf: &'a Bxdf,
    pub id: i32,
    pub medium: &'a Medium,
}

impl<'a> HitRecord<'a> {
    pub fn new() -> Self {
        HitRecord {
            distance: INF,
            hitpoint: Vec3::zero(),
            normal: Vec3::zero(),
            color: Vec3::zero(),
            bxdf: &Bxdf::Lambertian,
            id: -1,
            medium: &Medium {
                coeff_sc: Vec3(-1., -1., -1.),
                coeff_ex: Vec3(-1., -1., -1.),
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
            bxdf: &Bxdf::Lambertian,
            id: -1,
            medium: &Medium {
                coeff_sc: Vec3(-1., -1., -1.),
                coeff_ex: Vec3(-1., -1., -1.),
                g: 0.,
            },
        }
    }
}
