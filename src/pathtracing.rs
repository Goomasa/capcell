use crate::{
    material::*,
    math::{Color, PI, Vec3, dot, fmax, fmin, multiply},
    random::XorRand,
    ray::{HitRecord, Ray},
    scene::Scene,
};

const DEPTH: u32 = 6;
const MAX_DEPTH: u32 = 30;
const PI_INV: f64 = 1. / PI;

pub struct Pathtracing {
    record: HitRecord,
    now_ray: Ray,
    roulette_pdf: f64,
    orienting_normal: Vec3,
    throughput: Vec3,
    rad: Color,
    pt_sample_pdf: f64,
}

impl Pathtracing {
    pub fn new(ray: Ray) -> Self {
        Pathtracing {
            record: HitRecord::new(),
            now_ray: ray,
            roulette_pdf: 1.,
            orienting_normal: Vec3::new(0.),
            throughput: Vec3::new(1.),
            rad: Vec3::new(0.),
            pt_sample_pdf: -1.,
        }
    }

    fn roulette_prob(&self, time: u32) -> f64 {
        let mut prob = match self.record.bxdf {
            Bxdf::Light => 1.,
            _ => fmin(self.record.color.max_elm(), 1.),
        };

        if time > MAX_DEPTH {
            prob /= 2_i32.pow(time - MAX_DEPTH) as f64;
        } else if time <= DEPTH {
            prob = 1.;
        }

        prob
    }

    fn ray_intersect(&mut self, scene: &Scene) -> bool {
        self.record = HitRecord::new();
        if !scene.intersect_obj(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
            self.rad = self.rad + multiply(self.throughput, scene.background) / self.roulette_pdf;
            return false;
        }
        true
    }

    fn trace_light(&mut self, scene: &Scene) {
        if self.pt_sample_pdf < 0. {
            self.rad = self.rad + multiply(self.throughput, self.record.color) / self.roulette_pdf;
        } else {
            let nee_pdf = scene.pdf_sample_obj(&self.now_ray.org, &self.record);
            let mis_weight = self.pt_sample_pdf / (self.pt_sample_pdf + nee_pdf);
            self.rad = self.rad
                + multiply(self.throughput, self.record.color) * mis_weight / self.roulette_pdf;
        }
    }

    fn trace_lambertian(&mut self, scene: &Scene, rand: &mut XorRand) {
        let dir = sample_cos_hemisphere(&self.orienting_normal, rand);
        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        self.throughput = multiply(self.throughput, self.record.color);
        let nee_result = scene.nee(org, rand);

        if nee_result.pdf != 0. {
            let nee_dir_cos = fmax(dot(self.orienting_normal, nee_result.dir), 0.);
            let mis_weight = 1. / (nee_result.pdf + nee_dir_cos * PI_INV);
            self.rad = self.rad
                + multiply(self.throughput, nee_result.color) * PI_INV * nee_dir_cos * mis_weight
                    / self.roulette_pdf;
        }
        self.pt_sample_pdf = pdf_cos_hemisphere(&dir, &self.orienting_normal);
    }

    fn trace_microbrdf(&mut self, scene: &Scene, rand: &mut XorRand, ax: f64, ay: f64) {
        let wo = -self.now_ray.dir;
        let wm = sample_ggx_vndf(&self.orienting_normal, &wo, ax, ay, rand);
        let wi = reflection_dir(&wm, &self.now_ray.dir);
        let g1_wo = shadow_mask_fn(ax, ay, &wo, &self.orienting_normal);
        let fresnel = fresnel_color(&self.record.color, &wi, &wm);

        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir: wi };

        let nee_result = scene.nee(org, rand);
        let normal_dist = ggx_normal_df(ax, ay, &self.orienting_normal, &wm);
        let vndf = g1_wo * normal_dist / (4. * dot(wo, self.orienting_normal).abs());

        if nee_result.pdf != 0. {
            let nee_wm = (nee_result.dir + wo).normalize();
            let nee_normal_dist = ggx_normal_df(ax, ay, &self.orienting_normal, &nee_wm);
            let nee_vndf = g1_wo * nee_normal_dist / (4. * dot(wo, self.orienting_normal).abs());
            let mis_weight = 1. / (nee_result.pdf + nee_vndf);
            let nee_fresnel = fresnel_color(&self.record.color, &nee_result.dir, &nee_wm);

            let nee_g1_wi = shadow_mask_fn(ax, ay, &nee_result.dir, &self.orienting_normal);
            let brdf = nee_fresnel * nee_vndf * nee_g1_wi
                / dot(nee_result.dir, self.orienting_normal).abs();
            self.rad = self.rad
                + multiply(nee_result.color, multiply(self.throughput, brdf))
                    * dot(nee_result.dir, self.orienting_normal)
                    * mis_weight
                    / self.roulette_pdf;
        }

        let g1_wi = shadow_mask_fn(ax, ay, &wi, &self.orienting_normal);
        self.throughput = multiply(self.throughput, fresnel * g1_wi);
        if ax == 0. || ay == 0. {
            self.pt_sample_pdf = 1.;
        } else {
            self.pt_sample_pdf = vndf;
        }
    }

    fn trace_microbtdf(&mut self, scene: &Scene, rand: &mut XorRand, a: f64, ior: f64) {
        ()
    }

    pub fn integrate(&mut self, scene: &Scene, rand: &mut XorRand) -> Color {
        for time in 0.. {
            if !self.ray_intersect(scene) {
                break;
            }

            let roulette_prob = self.roulette_prob(time);
            if rand.next01() > roulette_prob {
                break;
            }
            self.roulette_pdf *= roulette_prob;

            self.orienting_normal = if dot(self.record.normal, self.now_ray.dir) < 0. {
                self.record.normal
            } else {
                -self.record.normal
            };

            match self.record.bxdf {
                Bxdf::Light => {
                    self.trace_light(scene);
                    break;
                }
                Bxdf::Lambertian => {
                    self.trace_lambertian(scene, rand);
                }
                Bxdf::MicroBrdf { ax, ay } => {
                    self.trace_microbrdf(scene, rand, ax, ay);
                }
                Bxdf::MicroBtdf { a, ior } => {
                    self.trace_microbtdf(scene, rand, a, ior);
                }
            }
        }
        self.rad
    }
}
