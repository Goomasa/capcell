use tobj::{Material, Model};

use crate::{
    material::{bxdf::Bxdf, medium::Medium},
    math::{Color, EPS, Vec3},
    object::Object,
    random::FreshId,
    texture::Texture,
};

#[allow(unused)]
pub fn load_obj<'a>(path: &str, scale: f64, freshid: &mut FreshId) -> Vec<Object<'a>> {
    let load_opt = {
        let mut default_opt = tobj::LoadOptions::default();
        default_opt.triangulate = true;
        default_opt
    };

    let load_result = tobj::load_obj(path, &load_opt);
    if let Err(msg) = load_result {
        println!("{}", msg);
        return Vec::new();
    }

    let load_result = load_result.unwrap();
    let models = load_result.0;

    let mut objs = Vec::new();
    let materials = if let Ok(mtl) = load_result.1 {
        let mut mats = mtl.iter().map(|m| load_mtl(m)).collect::<Vec<_>>();
        for _ in 0..models.len() - mtl.len() {
            mats.push((Bxdf::Lambertian, Vec3::new(1.), Medium::no_medium()));
        }
        mats
    } else {
        println!("failed to load mtl");
        vec![(Bxdf::Lambertian, Vec3::new(1.), Medium::no_medium()); models.len()]
    };

    for i in 0..models.len() {
        objs.append(&mut load_obj_model(
            &models[i],
            scale,
            &materials[i].0,
            &materials[i].1,
            &materials[i].2,
            freshid,
        ));
    }

    objs
}

fn load_obj_model<'a>(
    model: &Model,
    scale: f64,
    bxdf: &Bxdf,
    color: &Color,
    medium: &Medium,
    freshid: &mut FreshId,
) -> Vec<Object<'a>> {
    let mesh = &model.mesh;
    let mut verts = vec![Vec3::zero(); mesh.positions.len() / 3];
    for i in 0..mesh.positions.len() / 3 {
        let px = mesh.positions[i * 3] as f64 * scale;
        let py = mesh.positions[i * 3 + 1] as f64 * scale;
        let pz = mesh.positions[i * 3 + 2] as f64 * scale;
        verts[i] = Vec3(px, py, pz);
    }

    let mut normals = vec![Vec3::zero(); mesh.normals.len() / 3];
    for i in 0..mesh.normals.len() / 3 {
        let nx = mesh.normals[i * 3] as f64;
        let ny = mesh.normals[i * 3 + 1] as f64;
        let nz = mesh.normals[i * 3 + 2] as f64;
        normals[i] = Vec3(nx, ny, nz);
    }

    let mut objs = Vec::new();
    for i in 0..mesh.indices.len() / 3 {
        let v1 = verts[mesh.indices[i * 3] as usize];
        let v2 = verts[mesh.indices[i * 3 + 1] as usize];
        let v3 = verts[mesh.indices[i * 3 + 2] as usize];

        // assume mesh.indices.len()==mesh.normal_indices.len()
        // normal = simple mean of n1~3
        let normal = {
            let n1 = normals[mesh.normal_indices[i * 3] as usize];
            let n2 = normals[mesh.normal_indices[i * 3 + 1] as usize];
            let n3 = normals[mesh.normal_indices[i * 3 + 2] as usize];
            (n1 + n2 + n3) / 3.
        };

        let obj = Object::set_tri_with_normal(
            v1,
            v2,
            v3,
            normal.normalize(),
            *bxdf,
            Texture::set_solid(*color),
            freshid,
            *medium,
        );
        objs.push(obj);
    }

    objs
}

fn load_mtl(mtl: &Material) -> (Bxdf, Color, Medium) {
    let default_medium = Medium::no_medium();
    let emission = get_emission(mtl);
    if emission.length() > EPS {
        return (Bxdf::Light(emission), emission, default_medium);
    }

    let basecolor = if let Some(kd) = mtl.diffuse {
        Vec3(kd[0] as f64, kd[1] as f64, kd[2] as f64)
    } else {
        Vec3::new(1.)
    };

    let specular = if let Some(ks) = mtl.specular {
        (ks[0] + ks[1] + ks[2]) as f64 / 3.
    } else {
        1.
    };

    let metalic = if let Some(ns) = mtl.shininess {
        // ns=0 -> metal
        1. - (ns as f64 / 1000.)
    } else {
        0.
    };

    let roughness = 1. - metalic;

    let dissolve = if let Some(d) = mtl.dissolve { d } else { 1. };
    let (coeff_sc, coeff_ab, g) = get_medium_param(mtl);

    if dissolve < 0.5 || (coeff_ab.0 > 0. || coeff_sc.0 > 0.) {
        let ior = if let Some(ni) = mtl.optical_density {
            ni as f64
        } else {
            1.5
        };

        let medium = if coeff_sc.0 < 0. || coeff_ab.0 < 0. {
            default_medium
        } else {
            Medium::new(coeff_sc, coeff_ab, g)
        };

        if roughness < EPS {
            return (Bxdf::IdealGlass { ior }, basecolor, medium);
        } else {
            return (
                Bxdf::MicroBtdf {
                    a: roughness * roughness,
                    ior,
                },
                basecolor,
                medium,
            );
        }
    }

    if roughness > 0.99 {
        (Bxdf::Lambertian, basecolor, default_medium)
    } else {
        (
            Bxdf::set_comp(basecolor, metalic, specular, roughness),
            basecolor,
            default_medium,
        )
    }
}

fn get_emission(mtl: &Material) -> Color {
    let value = mtl.unknown_param.iter().find(|m| m.0 == "Ke");
    if let Some(ke) = value {
        let e = ke.1.split_whitespace().collect::<Vec<_>>();
        Vec3(
            e[0].parse::<f64>().unwrap_or(-1.),
            e[1].parse::<f64>().unwrap_or(-1.),
            e[2].parse::<f64>().unwrap_or(-1.),
        )
    } else {
        Vec3::new(-1.)
    }
}

fn get_medium_param(mtl: &Material) -> (Color, Color, f32) {
    // return (coeff_sc, coeff_ab, g)
    let value = mtl.unknown_param.iter().find(|m| m.0 == "sigma_sc");
    let coeff_sc = if let Some(sigma_sc) = value {
        let e = sigma_sc.1.split_whitespace().collect::<Vec<_>>();
        Vec3(
            e[0].parse::<f64>().unwrap_or(-1.),
            e[1].parse::<f64>().unwrap_or(-1.),
            e[2].parse::<f64>().unwrap_or(-1.),
        )
    } else {
        Vec3::new(-1.)
    };

    let value = mtl.unknown_param.iter().find(|m| m.0 == "sigma_ab");
    let coeff_ab = if let Some(sigma_ab) = value {
        let e = sigma_ab.1.split_whitespace().collect::<Vec<_>>();
        Vec3(
            e[0].parse::<f64>().unwrap_or(-1.),
            e[1].parse::<f64>().unwrap_or(-1.),
            e[2].parse::<f64>().unwrap_or(-1.),
        )
    } else {
        Vec3::new(-1.)
    };

    let value = mtl.unknown_param.iter().find(|m| m.0 == "g");
    let g = if let Some(g) = value {
        g.1.parse::<f32>().unwrap_or(0.)
    } else {
        0.
    };

    (coeff_sc, coeff_ab, g)
}
