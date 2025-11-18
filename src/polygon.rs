use tobj::Model;

use crate::{material::bxdf::Bxdf, math::Vec3, object::Object, random::FreshId, texture::Texture};

#[allow(unused)]
pub fn load_obj<'a>(path: &str, scale: f64, freshid: &mut FreshId) -> Vec<Object<'a>> {
    let load_result = tobj::load_obj(path, &tobj::LoadOptions::default());
    if let Err(_) = load_result {
        return Vec::new();
    }

    let mut objs = Vec::new();
    let models = load_result.unwrap().0;
    for model in models.iter() {
        objs.append(&mut load_obj_model(model, scale, freshid));
    }

    objs
}

fn load_obj_model<'a>(model: &Model, scale: f64, freshid: &mut FreshId) -> Vec<Object<'a>> {
    let mesh = &model.mesh;
    let mut verts = Vec::new();
    for i in 0..model.mesh.positions.len() / 3 {
        let px = mesh.positions[i * 3] as f64 * scale;
        let py = mesh.positions[i * 3 + 1] as f64 * scale;
        let pz = mesh.positions[i * 3 + 2] as f64 * scale;
        verts.push(Vec3(px, py, pz));
    }

    let mut objs = Vec::new();
    for i in 0..model.mesh.indices.len() / 3 {
        let v1 = verts[mesh.indices[i * 3] as usize];
        let v2 = verts[mesh.indices[i * 3 + 1] as usize];
        let v3 = verts[mesh.indices[i * 3 + 2] as usize];
        let obj = Object::set_tri(
            v1,
            v2,
            v3,
            Bxdf::Lambertian,
            Texture::set_solid(Vec3(0.1, 0.1, 1.)),
            freshid,
        );
        objs.push(obj);
    }

    objs
}
