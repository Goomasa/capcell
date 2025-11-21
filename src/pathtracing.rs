use crate::{
    Bxdf,
    material::{
        composite::{comp_specular_brdf, sample_brdf},
        diffuse::*,
        medium::{Medium, pdf_phase, pdfs_sample_distance, sample_phase, sample_rgb},
        microfacet::*,
    },
    math::{Color, EPS, INF, PI_INV, Vec3, dot, fmax, fmin},
    object::sphere_uv,
    random::XorRand,
    ray::{HitRecord, Ray},
    scene::Scene,
};

const DEPTH: u32 = 6;
const MAX_DEPTH: u32 = 15;

pub struct Pathtracing<'a> {
    record: HitRecord<'a>,
    now_ray: Ray,
    roulette_pdf: f64,
    orienting_normal: Vec3,
    throughput: Vec3,
    rad: Color,
    pt_sample_pdf: f64,
    medium_stack: Vec<(f64, &'a Medium)>, // (ior, medium)
}

impl<'a> Pathtracing<'a> {
    pub fn new(ray: Ray) -> Self {
        Pathtracing {
            record: HitRecord::new(),
            now_ray: ray,
            roulette_pdf: 1.,
            orienting_normal: Vec3::new(0.),
            throughput: Vec3::new(1.),
            rad: Vec3::new(0.),
            pt_sample_pdf: -1.,
            medium_stack: vec![(
                1.,
                &Medium {
                    coeff_sc: Vec3(-1., -1., -1.),
                    coeff_ex: Vec3(-1., -1., -1.),
                    g: 0.,
                },
            )],
        }
    }

    fn update_medium_stack(&mut self, into: bool, ior: f64, medium: &'a Medium) {
        if into {
            self.medium_stack.push((ior, medium));
        } else {
            if self.medium_stack.len() > 1 {
                self.medium_stack.pop();
            }
            // else: error
        }
    }

    fn get_coeff_ex(&self) -> Vec3 {
        self.medium_stack.last().unwrap().1.coeff_ex
    }

    fn get_ior(&self, ior: f64, into: bool) -> (f64, f64) {
        // return (ior_from, ior_to)
        if into {
            (ior, self.medium_stack.last().unwrap().0)
        } else {
            let size = self.medium_stack.len();
            if size <= 1 {
                (1., ior)
            } else {
                (self.medium_stack[size - 2].0, ior)
            }
        }
    }

    fn roulette_prob(&self, time: u32) -> f64 {
        let mut prob = match self.record.bxdf {
            Bxdf::Light(_) => 1.,
            _ => fmin(self.record.color.max_elm(), 1.),
        };

        if time > MAX_DEPTH {
            prob /= 2_i32.pow(time - MAX_DEPTH) as f64;
        } else if time <= DEPTH {
            prob = 1.;
        }

        prob
    }

    fn ray_intersect(&mut self, scene: &'a Scene) -> bool {
        self.record = HitRecord::new();
        if !scene.intersect_obj(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
            let (u, v) = sphere_uv(&Vec3::zero(), &self.now_ray.dir);
            self.rad =
                self.rad + self.throughput * scene.background.get_color(u, v) / self.roulette_pdf;
            return false;
        }
        true
    }

    fn trace_light(&mut self, emission: &Color, scene: &Scene) {
        if self.pt_sample_pdf < 0. {
            self.rad = self.rad + self.throughput * *emission / self.roulette_pdf;
        } else {
            let nee_pdf = scene.pdf_sample_obj(&self.now_ray.org, &self.record);
            let mis_weight = self.pt_sample_pdf / (self.pt_sample_pdf + nee_pdf);
            self.rad = self.rad + self.throughput * *emission * mis_weight / self.roulette_pdf;
        }
    }

    fn trace_lambertian(&mut self, scene: &Scene, rand: &mut XorRand) {
        let dir = sample_cos_hemisphere(&self.orienting_normal, rand);
        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        self.throughput = self.throughput * self.record.color;

        let nee_result = scene.nee(&org, &self.get_coeff_ex(), rand);
        if nee_result.pdf != 0. {
            let nee_dir_cos = fmax(dot(self.orienting_normal, nee_result.dir), 0.);
            let mis_weight = 1. / (nee_result.pdf + nee_dir_cos * PI_INV);
            self.rad = self.rad
                + self.throughput
                    * nee_result.color
                    * PI_INV
                    * nee_dir_cos
                    * mis_weight
                    * nee_result.transmittance
                    / self.roulette_pdf;
        }

        self.pt_sample_pdf = pdf_cos_hemisphere(&dir, &self.orienting_normal);
    }

    fn trace_mirror(&mut self) {
        let dir = reflection_dir(&self.orienting_normal, &self.now_ray.dir);
        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir };

        let fresnel = fresnel_color(&self.record.color, &dir, &self.orienting_normal);
        self.throughput = self.throughput * fresnel;
        self.pt_sample_pdf = INF;
    }

    fn trace_glass(&mut self, ior: f64, medium: &'a Medium, rand: &mut XorRand) {
        let into = dot(self.orienting_normal, self.record.normal) > 0.;
        let (ior_from, ior_to) = self.get_ior(ior, into);
        //let (ior_from, ior_to) = if into { (ior, 1.) } else { (1., ior) };

        let (is_refract, wi, _) = refraction_dir(
            ior_from,
            ior_to,
            &self.orienting_normal,
            &self.now_ray.dir,
            rand,
        );

        if is_refract {
            self.update_medium_stack(into, ior, medium);
            let org = self.record.hitpoint - 0.00001 * self.orienting_normal;
            self.now_ray = Ray { org, dir: wi };
            self.throughput = self.throughput * self.record.color;
            //self.roulette_pdf *= 1. - reflectance;
        } else {
            let org = self.record.hitpoint + 0.00001 * self.orienting_normal;
            self.now_ray = Ray { org, dir: wi };
            self.throughput = self.throughput * self.record.color;
            //self.roulette_pdf *= reflectance;
        }

        self.pt_sample_pdf = INF;
    }

    fn trace_microbrdf(&mut self, scene: &Scene, rand: &mut XorRand, ax: f64, ay: f64) {
        let wo = -self.now_ray.dir;
        let wm = sample_ggx_vndf(&self.orienting_normal, &wo, ax, ay, rand);
        let wi = reflection_dir(&wm, &self.now_ray.dir);
        let g1_wo = shadow_mask_fn(ax, ay, &wo, &self.orienting_normal);
        let fresnel = fresnel_color(&self.record.color, &wi, &wm);

        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        self.now_ray = Ray { org, dir: wi };

        let normal_dist = ggx_normal_df(ax, ay, &self.orienting_normal, &wm);
        let vndf = g1_wo * normal_dist / (4. * dot(wo, self.orienting_normal).abs());

        let nee_result = scene.nee(&org, &self.get_coeff_ex(), rand);
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
                + nee_result.color
                    * self.throughput
                    * brdf
                    * dot(nee_result.dir, self.orienting_normal)
                    * mis_weight
                    * nee_result.transmittance
                    / self.roulette_pdf;
        }

        let g1_wi = shadow_mask_fn(ax, ay, &wi, &self.orienting_normal);
        self.throughput = self.throughput * fresnel * g1_wi;
        if ax == 0. || ay == 0. {
            self.pt_sample_pdf = INF;
        } else {
            self.pt_sample_pdf = vndf;
        }
    }

    fn trace_microbtdf(
        &mut self,
        scene: &Scene,
        rand: &mut XorRand,
        a: f64,
        ior: f64,
        medium: &'a Medium,
    ) {
        let wo = -self.now_ray.dir;
        let wm = sample_ggx_vndf(&self.orienting_normal, &wo, a, a, rand);
        let g1_wo = shadow_mask_fn(a, a, &wo, &self.orienting_normal);
        let normal_dist = ggx_normal_df(a, a, &self.orienting_normal, &wm);

        let into = dot(self.record.normal, self.orienting_normal) > 0.;
        let (ior_from, ior_to) = self.get_ior(ior, into);

        let (is_refract, wi, reflectance) =
            refraction_dir(ior_from, ior_to, &wm, &self.now_ray.dir, rand);
        let g1_wi = shadow_mask_fn(a, a, &wi, &self.orienting_normal);

        let vndf;
        if is_refract {
            self.update_medium_stack(into, ior, medium);
            let org = self.record.hitpoint - self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir: wi };
            let j = micro_btdf_j(ior_from, ior_to, &wo, &wi, &wm);
            vndf =
                g1_wo * normal_dist * dot(wo, wm).abs() * j / dot(wo, self.orienting_normal).abs();

            let nee_result = scene.nee(&org, &self.get_coeff_ex(), rand);
            if nee_result.pdf != 0. {
                let nee_wm = -(wo * ior_to + nee_result.dir * ior_from).normalize();

                if dot(self.orienting_normal, nee_wm) > EPS {
                    let nee_j = micro_btdf_j(ior_from, ior_to, &wo, &nee_result.dir, &nee_wm);
                    let nee_normal_dist = ggx_normal_df(a, a, &self.orienting_normal, &nee_wm);
                    let nee_vndf = g1_wo * nee_normal_dist * dot(wo, nee_wm).abs() * nee_j
                        / dot(wo, self.orienting_normal).abs();
                    let mis_weight = 1. / (nee_result.pdf + nee_vndf);

                    let nee_g1_wi = shadow_mask_fn(a, a, &nee_result.dir, &self.orienting_normal);
                    let nee_fresnel = fresnel_ior(ior_from, ior_to, &wo, &nee_wm);
                    let btdf_ = (1. - nee_fresnel)
                        * nee_vndf
                        * nee_g1_wi
                        * (dot(nee_result.dir, nee_wm) / dot(wo, nee_wm)).abs();
                    self.rad = self.rad
                        + nee_result.color
                            * self.record.color
                            * self.throughput
                            * btdf_
                            * mis_weight
                            * nee_result.transmittance
                            / self.roulette_pdf;
                }
            }

            self.throughput = self.throughput * self.record.color * g1_wi * (1. - reflectance);
            self.roulette_pdf *= 1. - reflectance;
        } else {
            let org = self.record.hitpoint + self.orienting_normal * 0.00001;
            self.now_ray = Ray { org, dir: wi };
            vndf = g1_wo * normal_dist / (4. * dot(wo, self.orienting_normal).abs());

            let nee_result = scene.nee(&org, &self.get_coeff_ex(), rand);
            if nee_result.pdf != 0. {
                let nee_wm = (nee_result.dir + wo).normalize();
                let nee_normal_dist = ggx_normal_df(a, a, &self.orienting_normal, &nee_wm);
                let nee_vndf =
                    g1_wo * nee_normal_dist / (4. * dot(wo, self.orienting_normal).abs());
                let mis_weight = 1. / (nee_result.pdf + nee_vndf);
                let nee_fresnel = fresnel_color(&self.record.color, &nee_result.dir, &nee_wm);

                let nee_g1_wi = shadow_mask_fn(a, a, &nee_result.dir, &self.orienting_normal);
                let brdf = nee_fresnel * nee_vndf * nee_g1_wi
                    / dot(nee_result.dir, self.orienting_normal).abs();
                self.rad = self.rad
                    + nee_result.color
                        * self.throughput
                        * brdf
                        * dot(nee_result.dir, self.orienting_normal)
                        * mis_weight
                        * nee_result.transmittance
                        / self.roulette_pdf;
            }

            self.throughput = self.throughput * self.record.color * g1_wi * reflectance;
            self.roulette_pdf *= reflectance;
        }

        self.pt_sample_pdf = if a == 0. { INF } else { vndf };
    }

    pub fn trace_composite(
        &mut self,
        basecolor: &Color,
        metalic: f64,
        highlight: &Color,
        roughness: f64,
        scene: &Scene,
        rand: &mut XorRand,
    ) {
        let (is_diffuse, prob) = sample_brdf(metalic, rand);
        let org = self.record.hitpoint + self.orienting_normal * 0.00001;
        let wo = -self.now_ray.dir;

        let a = roughness * roughness;
        let wm;
        let wi;
        if is_diffuse {
            wi = sample_cos_hemisphere(&self.orienting_normal, rand);
            wm = (wi + wo).normalize();
        } else {
            wm = sample_ggx_vndf(&self.orienting_normal, &wo, a, a, rand);
            wi = reflection_dir(&wm, &self.now_ray.dir);
        }
        self.now_ray = Ray { org, dir: wi };

        let nee_result = scene.nee(&org, &self.get_coeff_ex(), rand);
        if nee_result.pdf != 0. {
            let nee_pdf_diffuse = pdf_cos_hemisphere(&nee_result.dir, &self.orienting_normal);
            let nee_wm = (wo + nee_result.dir).normalize();
            if is_diffuse {
                let pdf_pt = nee_pdf_diffuse;
                let mis_weight = prob / (pdf_pt + nee_result.pdf);
                self.rad = self.rad
                    + nee_result.color
                        * self.throughput
                        * *basecolor
                        * PI_INV
                        * dot(nee_result.dir, self.orienting_normal)
                        * mis_weight
                        * nee_result.transmittance
                        / self.roulette_pdf;
            } else {
                let (nee_pdf_specular, nee_brdf) = comp_specular_brdf(
                    a,
                    highlight,
                    &wo,
                    &nee_result.dir,
                    &nee_wm,
                    &self.orienting_normal,
                );
                let pdf_pt = nee_pdf_specular;
                let mis_weight = 1. / (pdf_pt + nee_result.pdf);
                self.rad = self.rad
                    + nee_result.color
                        * self.throughput
                        * nee_brdf
                        * dot(nee_result.dir, self.orienting_normal)
                        * mis_weight
                        * nee_result.transmittance
                        / self.roulette_pdf;
            }
        }

        let (pdf_diiffuse, brdf_diffuse) = (
            pdf_cos_hemisphere(&wi, &self.orienting_normal),
            *basecolor * PI_INV,
        );
        let (pdf_specular, brdf_specular) =
            comp_specular_brdf(a, highlight, &wo, &wi, &wm, &self.orienting_normal);
        let brdf = (1. - metalic) * brdf_diffuse + brdf_specular;
        self.pt_sample_pdf = prob * pdf_diiffuse + (1. - prob) * pdf_specular;

        if is_diffuse {
            self.throughput =
                self.throughput * brdf * dot(wi, self.orienting_normal) * prob / self.pt_sample_pdf;
        } else {
            self.throughput = self.throughput * brdf * dot(wi, self.orienting_normal) * (1. - prob)
                / self.pt_sample_pdf;
        }
    }

    pub fn freepath_sample(&mut self, scene: &'a Scene, rand: &mut XorRand) -> bool {
        let (_, medium) = self.medium_stack.last().unwrap();

        let pdf_rgb = self.throughput / self.throughput.sum();
        let sampled_coeff_ex = sample_rgb(&pdf_rgb, &medium.coeff_ex, rand);
        let dist = -1. * (1. - rand.next01()).ln() / sampled_coeff_ex;
        let pdf_sample_dist = pdfs_sample_distance(&medium.coeff_ex, dist);

        let mis_weight = pdf_sample_dist / (pdf_sample_dist * pdf_rgb).sum();
        self.throughput = self.throughput * mis_weight;

        self.record = HitRecord::init_with_distance(dist);
        if !scene.intersect_obj(&self.now_ray, &mut self.record, &scene.bvh_tree[0]) {
            self.throughput = self.throughput * (medium.coeff_sc / medium.coeff_ex);
            let org = self.now_ray.org + self.now_ray.dir * dist;
            let (dir, hg_pdf) = sample_phase(&-self.now_ray.dir, medium.g as f64, rand);

            let nee_result = scene.nee(&org, &medium.coeff_ex, rand);
            if nee_result.pdf != 0. {
                let nee_hg_pdf = pdf_phase(&nee_result.dir, medium.g as f64, &-self.now_ray.dir);
                let mis_weight = 1. / (nee_result.pdf + nee_hg_pdf);
                self.rad = self.rad
                    + self.throughput
                        * nee_result.color
                        * nee_hg_pdf
                        * nee_result.transmittance
                        * mis_weight
                        / self.roulette_pdf;
            }

            self.pt_sample_pdf = hg_pdf;
            self.now_ray = Ray { org, dir };
            return false;
        }
        true
    }

    pub fn integrate(&mut self, scene: &'a Scene, rand: &mut XorRand) -> Color {
        for time in 0.. {
            let roulette_prob = self.roulette_prob(time);
            if rand.next01() > roulette_prob {
                break;
            }
            self.roulette_pdf *= roulette_prob;

            if self.get_coeff_ex().0 > 0. {
                // in medium
                if !self.freepath_sample(scene, rand) {
                    continue;
                }
            } else {
                if !self.ray_intersect(scene) {
                    break;
                }
            }

            self.orienting_normal = if dot(self.record.normal, self.now_ray.dir) < 0. {
                self.record.normal
            } else {
                -self.record.normal
            };

            let medium = self.record.medium;

            match self.record.bxdf {
                Bxdf::NoSurface => {
                    let into = dot(self.orienting_normal, self.record.normal) > 0.;
                    self.update_medium_stack(into, 1., medium);
                    self.now_ray.org = self.record.hitpoint - 0.00001 * self.orienting_normal;
                }
                Bxdf::Light(emission) => {
                    if dot(self.record.normal, self.now_ray.dir) < 0. {
                        self.trace_light(emission, scene);
                    }
                    break;
                }
                Bxdf::Lambertian => {
                    self.trace_lambertian(scene, rand);
                }
                Bxdf::IdealMirror => {
                    self.trace_mirror();
                }
                Bxdf::IdealGlass { ior } => {
                    self.trace_glass(*ior, medium, rand);
                }
                Bxdf::MicroBrdf { ax, ay } => {
                    self.trace_microbrdf(scene, rand, *ax, *ay);
                }
                Bxdf::MicroBtdf { a, ior } => {
                    self.trace_microbtdf(scene, rand, *a, *ior, medium);
                }
                Bxdf::CompositeBrdf {
                    basecolor,
                    metalic,
                    highlight,
                    roughness,
                } => {
                    self.trace_composite(&basecolor, *metalic, &highlight, *roughness, scene, rand);
                }
            }
        }
        self.rad
    }
}
