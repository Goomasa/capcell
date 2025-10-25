use crate::{
    bvh::{BvhNode, BvhTree, construct_bvh},
    math::{Color, Point3, Vec3},
    object::{
        Object, sample_rect, sample_rect_pdf, sample_sphere, sample_sphere_pdf, sample_tri_pdf,
        sample_triangle,
    },
    random::XorRand,
    ray::{HitRecord, Ray},
};

pub struct NeeResult {
    pub dir: Vec3,
    pub color: Color,
    pub pdf: f64,
}

impl NeeResult {
    pub fn new() -> Self {
        NeeResult {
            dir: Vec3::zero(),
            color: Color::zero(),
            pdf: 0.,
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
        let bvh_tree = construct_bvh(&objs);

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
            Object::Sphere { center, radius, .. } => sample_sphere_pdf(org, center, *radius),
            Object::Rectangle { .. } => sample_rect_pdf(org, &record.hitpoint, obj, record.normal),
            Object::Triangle { .. } => sample_tri_pdf(org, &record.hitpoint, obj, record.normal),
        }
    }

    pub fn nee(&self, org: Point3, rand: &mut XorRand) -> NeeResult {
        let mut nee_result = NeeResult::new();
        let size = self.lights.len() as u32;
        if size == 0 {
            return nee_result;
        }

        let idx = rand.nexti() % size;
        let obj = self.lights[idx as usize];

        let (pdf, dir, _) = match obj {
            Object::Sphere { center, radius, .. } => sample_sphere(org, center, *radius, rand),
            Object::Rectangle {
                axis, min_p, max_p, ..
            } => sample_rect(org, axis, max_p, min_p, rand),
            Object::Triangle {
                p, pq, pr, normal, ..
            } => sample_triangle(org, p, pq, pr, normal, obj.get_area(), rand),
        };

        let mut record = HitRecord::new();
        let ray = Ray { org, dir };
        let _ = self.intersect_obj(&ray, &mut record, &self.bvh_tree[0]);
        if record.id != obj.get_obj_id() {
            return nee_result;
        }

        nee_result.dir = dir;
        nee_result.color = record.color;
        nee_result.pdf = pdf / size as f64;

        nee_result
    }
}
