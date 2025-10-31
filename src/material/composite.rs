use crate::{
    material::microfacet::{fresnel_color, ggx_normal_df, shadow_mask_fn},
    math::{Color, Vec3, dot},
    random::XorRand,
};

pub fn comp_specular_brdf(
    a: f64,
    highlight: &Color,
    wo: &Vec3,
    wi: &Vec3,
    wm: &Vec3,
    normal: &Vec3,
) -> (f64, Vec3) {
    let g1_wo = shadow_mask_fn(a, a, wo, normal);
    let normal_dist = ggx_normal_df(a, a, normal, wm);
    let vndf = normal_dist * g1_wo / (4. * dot(*wo, *normal).abs());

    let g1_wi = shadow_mask_fn(a, a, wi, normal);
    let fresnl = fresnel_color(highlight, wi, wm);

    (vndf, vndf * fresnl * g1_wi / dot(*wi, *normal).abs())
}

pub fn sample_brdf(metalic: f64, rand: &mut XorRand) -> (bool, f64) {
    // probability of sampling diffuse
    let prob = (1. - metalic) / ((1. - metalic) + 1.);
    (rand.next01() < prob, prob)
}
