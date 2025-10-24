use crate::math::{Point3, Vec3};

pub struct Ray {
    pub org: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(org: Point3, dir: Vec3) -> Self {
        Ray { org, dir }
    }
}
