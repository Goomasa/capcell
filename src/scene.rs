use crate::{
    bvh::{BvhNode, BvhTree, construct_bvh},
    material::bxdf::Bxdf,
    math::{Color, Point3, Vec3, dot},
    object::{Object, pdf_sample_rect, pdf_sample_sphere, pdf_sample_tri},
    random::XorRand,
    ray::{HitRecord, Ray},
};

pub struct NeeResult {
    pub dir: Vec3,
    pub color: Color,
    pub pdf: f64,
    pub transmittance: f64,
}

impl NeeResult {
    pub fn new() -> Self {
        NeeResult {
            dir: Vec3::zero(),
            color: Color::zero(),
            pdf: 0.,
            transmittance: 1.,
        }
    }
}

pub struct Scene<'a> {
    pub objects: Vec<&'a Object>,
    pub background: Color,
    pub lights: Vec<&'a Object>,
    pub bvh_tree: BvhTree,
}

impl<'a> Scene<'a> {
    pub fn new(mut objs: Vec<&'a Object>, back: Color) -> Self {
        objs.shrink_to_fit();
        let lights = objs
            .clone()
            .into_iter()
            .filter(|obj| obj.get_bxdf().is_light())
            .collect();

        objs.sort_by(|o1, o2| o1.get_obj_id().cmp(&o2.get_obj_id()));
        let mut bvh_tree = construct_bvh(&objs);
        bvh_tree.shrink_to_fit();

        Scene {
            objects: objs,
            background: back,
            lights,
            bvh_tree,
        }
    }

    pub fn intersect_obj(&self, ray: &Ray, record: &mut HitRecord, node: &BvhNode) -> bool {
        let (l, r) = node.children;
        if node.bbox.hit(ray, record) {
            if l == -1 {
                for i in node.elements.iter() {
                    let _ = self.objects[*i].hit(ray, record);
                }
            } else {
                let _ = self.intersect_obj(ray, record, &self.bvh_tree[l as usize]);
                let _ = self.intersect_obj(ray, record, &self.bvh_tree[r as usize]);
            }
        }
        record.id != -1
    }

    pub fn pdf_sample_obj(&self, org: &Point3, record: &HitRecord) -> f64 {
        let obj = self.objects[record.id as usize];
        match obj {
            Object::Sphere { center, radius, .. } => pdf_sample_sphere(org, center, *radius),
            Object::Rectangle { .. } => pdf_sample_rect(org, &record.hitpoint, obj, record.normal),
            Object::Triangle { .. } => pdf_sample_tri(org, &record.hitpoint, obj, record.normal),
        }
    }

    pub fn calc_transmittance(
        &self,
        ray: &mut Ray,
        coeff_ex: f64,
        light_id: i32,
        distance: f64,
    ) -> (f64, Color) {
        // retrun (transmittance, emission)
        let mut has_medium = coeff_ex > 0.;
        let mut now_coeff = if has_medium { coeff_ex } else { 0. };
        let mut transmittance = 1.;
        let mut record;

        loop {
            record = HitRecord::init_with_distance(distance + 0.1);
            if !self.intersect_obj(ray, &mut record, &self.bvh_tree[0]) {
                return (-1., Vec3::zero());
            }

            if record.id == light_id && dot(ray.dir, record.normal) < 0. {
                if has_medium {
                    transmittance *= (-record.distance * now_coeff).exp();
                }
                //println!("{}, {}", transmittance, coeff_ex);
                return (transmittance, record.color);
            }

            if let Bxdf::NoSurface = record.bxdf {
                if has_medium {
                    transmittance *= (-record.distance * now_coeff).exp();
                }
                has_medium = !has_medium;
                now_coeff = record.medium.coeff_ex;
                ray.org = record.hitpoint + 0.00001 * ray.dir;
            } else {
                return (-1., Vec3::zero());
            }
        }
    }

    pub fn nee(&self, org: &Point3, coeff_ex: f64, rand: &mut XorRand) -> NeeResult {
        let mut nee_result = NeeResult::new();
        let size = self.lights.len() as u32;
        if size == 0 {
            return nee_result;
        }

        let idx = rand.nexti() % size;
        let light = self.lights[idx as usize];

        let (pdf, dir, distance) = light.pdf_sample_surface(org, rand);
        let mut ray = Ray { org: *org, dir };

        let (transmittance, emission) =
            self.calc_transmittance(&mut ray, coeff_ex, light.get_obj_id(), distance);
        if transmittance < 0. {
            return nee_result;
        }

        nee_result.dir = dir;
        nee_result.color = emission;
        nee_result.pdf = pdf / size as f64;
        nee_result.transmittance = transmittance;

        nee_result
    }
}
