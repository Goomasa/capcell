use crate::aabb::AABB;
use crate::material::medium::Medium;
use crate::random::{FreshId, XorRand};
use crate::ray::*;
use crate::texture::Texture;
use crate::{material::bxdf::Bxdf, math::*};

#[derive(Clone, Copy)]
pub enum Axis {
    X(bool),
    Y(bool),
    Z(bool),
}

#[allow(unused)]
pub enum Object<'a> {
    Sphere {
        center: Point3,
        radius: f64,
        bxdf: Bxdf,
        texture: Texture<'a>,
        obj_id: i32,
        bbox: AABB,
        medium: Medium,
    },

    Rectangle {
        axis: Axis,
        min_p: Point3,
        max_p: Point3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        obj_id: i32,
        bbox: AABB,
        medium: Medium,
    },

    Triangle {
        p: Point3,
        pq: Point3,
        pr: Point3,
        normal: Vec3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        obj_id: i32,
        bbox: AABB,
        medium: Medium,
    },
}

#[allow(unused)]
impl<'a> Object<'a> {
    pub fn set_sphere(
        center: Point3,
        radius: f64,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
    ) -> Object<'a> {
        Object::Sphere {
            center,
            radius,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB {
                min_p: center - Vec3::new(radius),
                max_p: center + Vec3::new(radius),
            },
            medium: Medium {
                coeff_sc: -1.,
                coeff_ex: -1.,
                g: 0.,
            },
        }
    }

    pub fn set_sphere_with_medium(
        center: Point3,
        radius: f64,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
        medium: Medium,
    ) -> Object<'a> {
        Object::Sphere {
            center,
            radius,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB {
                min_p: center - Vec3::new(radius),
                max_p: center + Vec3::new(radius),
            },
            medium,
        }
    }

    pub fn set_rect(
        axis: Axis,
        p: Point3,
        q: Point3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
    ) -> Object<'a> {
        let max_p = Vec3(fmax(p.0, q.0), fmax(p.1, q.1), fmax(p.2, q.2));
        let min_p = Vec3(fmin(p.0, q.0), fmin(p.1, q.1), fmin(p.2, q.2));

        Object::Rectangle {
            axis,
            min_p,
            max_p,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.fix_aabb(),
            medium: Medium {
                coeff_sc: -1.,
                coeff_ex: -1.,
                g: 0.,
            },
        }
    }

    pub fn set_rect_with_medium(
        axis: Axis,
        p: Point3,
        q: Point3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
        medium: Medium,
    ) -> Object<'a> {
        let max_p = Vec3(fmax(p.0, q.0), fmax(p.1, q.1), fmax(p.2, q.2));
        let min_p = Vec3(fmin(p.0, q.0), fmin(p.1, q.1), fmin(p.2, q.2));

        Object::Rectangle {
            axis,
            min_p,
            max_p,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.fix_aabb(),
            medium,
        }
    }

    pub fn set_tri(
        p: Point3,
        q: Point3,
        r: Point3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
    ) -> Object<'a> {
        let normal = cross(q - p, r - p).normalize();
        let min_p = Vec3(
            fmin(p.0, fmin(q.0, r.0)),
            fmin(p.1, fmin(q.1, r.1)),
            fmin(p.2, fmin(q.2, r.2)),
        );
        let max_p = Vec3(
            fmax(p.0, fmax(q.0, r.0)),
            fmax(p.1, fmax(q.1, r.1)),
            fmax(p.2, fmax(q.2, r.2)),
        );
        Object::Triangle {
            p,
            pq: q - p,
            pr: r - p,
            normal,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.fix_aabb(),
            medium: Medium {
                coeff_sc: -1.,
                coeff_ex: -1.,
                g: 0.,
            },
        }
    }

    pub fn set_tri_with_medium(
        p: Point3,
        q: Point3,
        r: Point3,
        bxdf: Bxdf,
        texture: Texture<'a>,
        freshid: &mut FreshId,
        medium: Medium,
    ) -> Object<'a> {
        let normal = cross(q - p, r - p).normalize();
        let min_p = Vec3(
            fmin(p.0, fmin(q.0, r.0)),
            fmin(p.1, fmin(q.1, r.1)),
            fmin(p.2, fmin(q.2, r.2)),
        );
        let max_p = Vec3(
            fmax(p.0, fmax(q.0, r.0)),
            fmax(p.1, fmax(q.1, r.1)),
            fmax(p.2, fmax(q.2, r.2)),
        );
        Object::Triangle {
            p,
            pq: q - p,
            pr: r - p,
            normal,
            bxdf,
            texture,
            obj_id: freshid.gen_id(),
            bbox: AABB { min_p, max_p }.fix_aabb(),
            medium,
        }
    }

    pub fn hit(&self, ray: &Ray, record: &mut HitRecord) -> bool {
        match self {
            Object::Sphere {
                center,
                radius,
                bxdf,
                texture,
                obj_id: id,
                medium,
                ..
            } => {
                if let Some((t, hitpoint, normal)) =
                    hit_sphere(center, radius, ray, record.distance)
                {
                    record.distance = t;
                    record.hitpoint = hitpoint;
                    record.normal = normal;
                    record.bxdf = *bxdf;
                    record.color = {
                        let (u, v) = sphere_uv(center, &hitpoint);
                        texture.get_color(u, v)
                    };
                    record.id = *id;
                    record.medium = *medium;
                    true
                } else {
                    false
                }
            }
            Object::Rectangle {
                axis,
                min_p,
                max_p,
                bxdf,
                texture,
                obj_id: id,
                medium,
                ..
            } => {
                if let Some((t, hitpoint, normal, (u, v))) =
                    hit_rect(axis, max_p, min_p, ray, record.distance)
                {
                    record.distance = t;
                    record.hitpoint = hitpoint;
                    record.normal = normal;
                    record.bxdf = *bxdf;
                    record.color = texture.get_color(u, v);
                    record.id = *id;
                    record.medium = *medium;
                    true
                } else {
                    false
                }
            }
            Object::Triangle {
                p,
                pq,
                pr,
                normal,
                bxdf,
                texture,
                obj_id: id,
                medium,
                ..
            } => {
                if let Some((t, hitpoint)) = hit_triangle(p, pq, pr, normal, ray, record.distance) {
                    record.distance = t;
                    record.hitpoint = hitpoint;
                    record.normal = *normal;
                    record.bxdf = *bxdf;
                    record.color = texture.get_color(0., 0.);
                    record.id = *id;
                    record.medium = *medium;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn pdf_sample_surface(&self, org: &Point3, rand: &mut XorRand) -> (f64, Vec3, f64) {
        match self {
            Object::Sphere { center, radius, .. } => sample_sphere(*org, center, *radius, rand),
            Object::Rectangle {
                axis, min_p, max_p, ..
            } => sample_rect(*org, axis, max_p, min_p, rand),
            Object::Triangle {
                p, pq, pr, normal, ..
            } => sample_triangle(*org, p, pq, pr, normal, self.get_area(), rand),
        }
    }

    pub fn get_bxdf(&self) -> &Bxdf {
        match self {
            Object::Sphere { bxdf, .. }
            | Object::Rectangle { bxdf, .. }
            | Object::Triangle { bxdf, .. } => bxdf,
        }
    }

    pub fn get_obj_id(&self) -> i32 {
        match self {
            Object::Sphere { obj_id: id, .. }
            | Object::Rectangle { obj_id: id, .. }
            | Object::Triangle { obj_id: id, .. } => *id,
        }
    }

    pub fn get_bbox(&self) -> &AABB {
        match self {
            Object::Sphere { bbox, .. }
            | Object::Rectangle { bbox, .. }
            | Object::Triangle { bbox, .. } => bbox,
        }
    }

    pub fn get_area(&self) -> f64 {
        match self {
            Object::Sphere { radius, .. } => 2. * PI * radius,
            Object::Rectangle {
                axis, min_p, max_p, ..
            } => match axis {
                Axis::X(_) => (max_p.1 - min_p.1) * (max_p.2 - min_p.2),
                Axis::Y(_) => (max_p.0 - min_p.0) * (max_p.2 - min_p.2),
                Axis::Z(_) => (max_p.0 - min_p.0) * (max_p.1 - min_p.1),
            },
            Object::Triangle { pq, pr, .. } => cross(*pq, *pr).length() / 2.,
        }
    }

    pub fn get_center(&self) -> Point3 {
        let bbox = self.get_bbox();
        (bbox.min_p + bbox.max_p) / 2.
    }
}

pub fn sphere_uv(center: &Point3, pos: &Point3) -> (f64, f64) {
    let op = (*pos - *center).normalize();
    let theta = dot(Vec3(0., 1., 0.), op).acos(); // 0 - PI
    let u_op = dot(Vec3(0., 0., 1.), op);

    let mut phi; // 0 - 2PI
    if u_op < EPS && u_op >= 0. {
        phi = PI / 2.;
    } else if u_op > -EPS && u_op < 0. {
        phi = 3. * PI / 2.
    } else if PI - theta < EPS || theta + PI < EPS {
        phi = 0.
    } else {
        phi = (dot(Vec3(1., 0., 0.), op) / u_op).atan();
        let cos_phi = u_op / theta.sin();
        if cos_phi < 0. && phi < 0. {
            phi = PI + phi;
        } else if cos_phi < 0. && phi > 0. {
            phi = PI + phi;
        } else if cos_phi > 0. && phi < 0. {
            phi = 2. * PI + phi;
        }
    }

    (phi / (2. * PI), theta / PI)
}

fn hit_sphere(
    center: &Point3,
    radius: &f64,
    ray: &Ray,
    max_dist: f64,
) -> Option<(f64, Point3, Vec3)> {
    //if hit, return (distant, hitpoint, normal)
    let oc = *center - ray.org;
    let oc_dir = dot(oc, ray.dir);
    let disc = oc_dir * oc_dir - oc.length_sq() + radius * radius;

    if disc < 0. {
        return None;
    }

    let t1 = oc_dir - disc.sqrt();
    let t2 = oc_dir + disc.sqrt();
    let t;

    if t1 > 0. {
        t = t1;
    } else if t2 > 0. {
        t = t2;
    } else {
        return None;
    }

    if t > max_dist {
        return None;
    }

    Some((t, ray.org + ray.dir * t, (ray.dir * t - oc).normalize()))
}

pub fn hit_rect(
    axis: &Axis,
    max_p: &Point3,
    min_p: &Point3,
    ray: &Ray,
    max_dist: f64,
) -> Option<(f64, Point3, Vec3, (f64, f64))> {
    let hitpoint;
    let diff = *max_p - *min_p;
    let t;
    match axis {
        Axis::X(face) => {
            if ray.dir.0.abs() < EPS {
                return None;
            }

            t = (max_p.0 - ray.org.0) / ray.dir.0;
            if t < 0. || t > max_dist {
                return None;
            }

            hitpoint = ray.org + ray.dir * t;

            if hitpoint.1 > max_p.1
                || hitpoint.1 < min_p.1
                || hitpoint.2 > max_p.2
                || hitpoint.2 < min_p.2
            {
                return None;
            } else {
                let hd = hitpoint - *min_p;
                return Some((
                    t,
                    hitpoint,
                    Vec3(get_face(*face), 0., 0.),
                    (hd.2 / diff.2, hd.1 / diff.1),
                ));
            }
        }
        Axis::Y(face) => {
            if ray.dir.1.abs() < EPS {
                return None;
            }

            t = (max_p.1 - ray.org.1) / ray.dir.1;
            if t < 0. || t > max_dist {
                return None;
            }

            hitpoint = ray.org + ray.dir * t;

            if hitpoint.0 > max_p.0
                || hitpoint.0 < min_p.0
                || hitpoint.2 > max_p.2
                || hitpoint.2 < min_p.2
            {
                return None;
            } else {
                let hd = hitpoint - *min_p;
                return Some((
                    t,
                    hitpoint,
                    Vec3(0., get_face(*face), 0.),
                    (hd.0 / diff.0, hd.2 / diff.2),
                ));
            }
        }
        Axis::Z(face) => {
            if ray.dir.2.abs() < EPS {
                return None;
            }

            t = (max_p.2 - ray.org.2) / ray.dir.2;
            if t < 0. || t > max_dist {
                return None;
            }

            hitpoint = ray.org + ray.dir * t;

            if hitpoint.0 > max_p.0
                || hitpoint.0 < min_p.0
                || hitpoint.1 > max_p.1
                || hitpoint.1 < min_p.1
            {
                return None;
            } else {
                let hd = hitpoint - *min_p;
                return Some((
                    t,
                    hitpoint,
                    Vec3(0., 0., get_face(*face)),
                    (hd.0 / diff.0, hd.1 / diff.1),
                ));
            }
        }
    }
}

pub fn hit_triangle(
    p: &Point3,
    pq: &Point3,
    pr: &Point3,
    normal: &Vec3,
    ray: &Ray,
    max_dist: f64,
) -> Option<(f64, Point3)> {
    let n_d = dot(*normal, ray.dir);
    if n_d == 0. {
        return None;
    }

    let t = dot(*normal, *p - ray.org) / n_d;
    if t > max_dist || t <= 0. {
        return None;
    }
    let pos = ray.org + ray.dir * t;
    let p_pos = pos - *p;
    let u = dot(cross(*pr, p_pos), *normal) / dot(cross(*pr, *pq), *normal);
    let v = dot(cross(*pq, p_pos), *normal) / dot(cross(*pq, *pr), *normal);

    if u + v > 1. || u < 0. || v < 0. {
        return None;
    }

    Some((t, pos))
}

pub fn sample_sphere(
    org: Point3,
    center: &Point3,
    radius: f64,
    rand: &mut XorRand,
) -> (f64, Vec3, f64) {
    let pc = *center - org;
    let cos_mu = (1. - (radius * radius / pc.length_sq())).sqrt();

    let w = pc.normalize();
    let u = if w.0 > EPS || w.0 < (-EPS) {
        cross(w, Vec3(0., 1., 0.)).normalize()
    } else {
        cross(w, Vec3(1., 0., 0.)).normalize()
    };
    let v = cross(w, u);

    let phi = 2. * PI * rand.next01();
    let cos_theta = 1. - rand.next01() * (1. - cos_mu);
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();

    let dir = u * sin_theta * phi.cos() + v * sin_theta * phi.sin() + w * cos_theta;
    let pdf = 1. / (2. * PI * (1. - cos_mu));
    (pdf, dir, pc.length())
}

pub fn sample_rect(
    org: Point3,
    axis: &Axis,
    max_p: &Point3,
    min_p: &Point3,
    rand: &mut XorRand,
) -> (f64, Vec3, f64) {
    let diagnal = *max_p - *min_p;
    let area;
    let normal;
    let oa;
    let ob;
    match axis {
        Axis::X(_) => {
            area = diagnal.1 * diagnal.2;
            normal = Vec3(1., 0., 0.);
            oa = Vec3(0., diagnal.1, 0.);
            ob = Vec3(0., 0., diagnal.2);
        }
        Axis::Y(_) => {
            area = diagnal.0 * diagnal.2;
            normal = Vec3(0., 1., 0.);
            oa = Vec3(diagnal.0, 0., 0.);
            ob = Vec3(0., 0., diagnal.2);
        }
        Axis::Z(_) => {
            area = diagnal.0 * diagnal.1;
            normal = Vec3(0., 0., 1.);
            oa = Vec3(diagnal.0, 0., 0.);
            ob = Vec3(0., diagnal.1, 0.);
        }
    }

    let sample_pos = *min_p + oa * rand.next01() + ob * rand.next01();
    let mut dir = sample_pos - org;
    let l_sq = dir.length_sq();
    dir = dir.normalize();
    let cos_theta = dot(dir, normal).abs();
    (l_sq / (area * cos_theta), dir, l_sq.sqrt())
}

pub fn sample_triangle(
    org: Point3,
    p: &Point3,
    pq: &Point3,
    pr: &Point3,
    normal: &Vec3,
    area: f64,
    rand: &mut XorRand,
) -> (f64, Vec3, f64) {
    let mut r1 = rand.next01();
    let mut r2 = rand.next01();
    if r1 + r2 > 1. {
        r1 = 1. - r1;
        r2 = 1. - r2;
    }
    let sample_pos = *p + *pq * r1 + *pr * r2;
    let mut dir = sample_pos - org;

    let l_sq = dir.length_sq();
    dir = dir.normalize();
    let cos_theta = dot(dir, *normal).abs();
    (l_sq / (cos_theta * area), dir, l_sq.sqrt())
}

pub fn pdf_sample_sphere(org: &Point3, center: &Point3, radius: f64) -> f64 {
    let cos_mu = (1. - (radius * radius / (*center - *org).length_sq())).sqrt();
    1. / (2. * PI * (1. - cos_mu))
}

pub fn pdf_sample_rect(org: &Point3, pos: &Point3, obj: &Object, normal: Vec3) -> f64 {
    let l_sq = (*pos - *org).length_sq();
    let cos_theta = dot((*pos - *org).normalize(), normal).abs();
    l_sq / (obj.get_area() * cos_theta)
}

pub fn pdf_sample_tri(org: &Point3, pos: &Point3, obj: &Object, normal: Vec3) -> f64 {
    let l_sq = (*pos - *org).length_sq();
    let cos_theta = dot((*pos - *org).normalize(), normal).abs();
    l_sq / (obj.get_area() * cos_theta)
}

fn get_face(face: bool) -> f64 {
    if face { 1. } else { -1. }
}
