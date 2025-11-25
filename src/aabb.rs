use crate::{
    math::{INF, Point3, Vec3, fmax, fmin},
    object::Object,
    ray::{HitRecord, Ray},
};

use std::ops::Add;

pub struct AABB {
    pub min_p: Point3,
    pub max_p: Point3,
}

impl Add<&Self> for AABB {
    type Output = Self;
    fn add(self, rhs: &Self) -> Self::Output {
        let max_p = Vec3(
            fmax(self.max_p.0, rhs.max_p.0),
            fmax(self.max_p.1, rhs.max_p.1),
            fmax(self.max_p.2, rhs.max_p.2),
        );
        let min_p = Vec3(
            fmin(self.min_p.0, rhs.min_p.0),
            fmin(self.min_p.1, rhs.min_p.1),
            fmin(self.min_p.2, rhs.min_p.2),
        );

        AABB { min_p, max_p }
    }
}

impl AABB {
    pub fn empty() -> Self {
        AABB {
            min_p: Vec3::new(INF),
            max_p: Vec3::new(-INF),
        }
    }

    pub fn hit(&self, ray: &Ray, record: &mut HitRecord) -> bool {
        if ray.dir.0 != 0. {
            let t1 = (self.min_p.0 - ray.org.0) / ray.dir.0;
            let t2 = (self.max_p.0 - ray.org.0) / ray.dir.0;
            if (t1 < 0. && t2 < 0.) || fmin(t1, t2) > record.distance {
                return false;
            }

            let p1 = ray.org + ray.dir * t1;
            let p2 = ray.org + ray.dir * t2;

            if (p1.1 < self.min_p.1 && p2.1 < self.min_p.1)
                || (p1.1 > self.max_p.1 && p2.1 > self.max_p.1)
                || (p1.2 < self.min_p.2 && p2.2 < self.min_p.2)
                || (p1.2 > self.max_p.2 && p2.2 > self.max_p.2)
            {
                return false;
            }
            true
        } else if ray.dir.1 != 0. {
            let t1 = (self.min_p.1 - ray.org.1) / ray.dir.1;
            let t2 = (self.max_p.1 - ray.org.1) / ray.dir.1;
            if (t1 < 0. && t2 < 0.) || fmin(t1, t2) > record.distance {
                return false;
            }

            let p1 = ray.org + ray.dir * t1;
            let p2 = ray.org + ray.dir * t2;

            if (p1.0 < self.min_p.0 && p2.0 < self.min_p.0)
                || (p1.0 > self.max_p.0 && p2.0 > self.max_p.0)
                || (p1.2 < self.min_p.2 && p2.2 < self.min_p.2)
                || (p1.2 > self.max_p.2 && p2.2 > self.max_p.2)
            {
                return false;
            }
            true
        } else if ray.dir.2 != 0. {
            let t1 = (self.min_p.2 - ray.org.2) / ray.dir.2;
            let t2 = (self.max_p.2 - ray.org.2) / ray.dir.2;
            if (t1 < 0. && t2 < 0.) || fmin(t1, t2) > record.distance {
                return false;
            }

            let p1 = ray.org + ray.dir * t1;
            let p2 = ray.org + ray.dir * t2;

            if (p1.1 < self.min_p.1 && p2.1 < self.min_p.1)
                || (p1.1 > self.max_p.1 && p2.1 > self.max_p.1)
                || (p1.0 < self.min_p.0 && p2.0 < self.min_p.0)
                || (p1.0 > self.max_p.0 && p2.0 > self.max_p.0)
            {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn get_area(&self) -> f64 {
        let diff = self.max_p - self.min_p;
        2. * ((diff.0 + diff.1) * diff.2 + diff.0 * diff.1)
    }

    pub fn fix(&self) -> AABB {
        let mut max_p = self.max_p;
        let mut min_p = self.min_p;
        if self.max_p.0 == self.min_p.0 {
            min_p.0 -= 0.01;
            max_p.0 += 0.01;
        }
        if self.max_p.1 == self.min_p.1 {
            min_p.1 -= 0.01;
            max_p.1 += 0.01;
        }
        if self.max_p.2 == self.min_p.2 {
            min_p.2 -= 0.01;
            max_p.2 += 0.01;
        }
        AABB { min_p, max_p }
    }

    pub fn entire_box(objs: &Vec<&Object>) -> Self {
        let mut bbox = AABB::empty();
        for obj in objs {
            bbox = bbox + &obj.bbox;
        }
        bbox
    }
}
