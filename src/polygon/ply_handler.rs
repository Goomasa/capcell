use std::{fs::File, io::Read};

use crate::{
    material::{bxdf::Bxdf, medium::Medium},
    math::{Color, Vec3},
    object::Object,
    random::FreshId,
    texture::Texture,
};

pub fn parse_ply(path: &str) -> (Vec<f64>, Vec<usize>) {
    let mut coords = Vec::new();
    let mut ids = Vec::new();

    let mut file = File::open(path).expect("file not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("can not read");
    let contents = contents.split_whitespace().collect::<Vec<_>>();

    let mut coord_from = contents.len();
    let mut id_from = contents.len();
    for i in 0..contents.len() {
        if contents[i] == "end_header" {
            coord_from = i + 1;
        }
        if contents[i] == "3" {
            id_from = i;
            break;
        }
    }

    for i in coord_from..id_from {
        if let Ok(f) = contents[i].parse::<f64>() {
            coords.push(f);
        }
    }

    for i in id_from..contents.len() {
        if let Ok(u) = contents[i].parse::<usize>() {
            ids.push(u);
        }
    }

    (coords, ids)
}

// only for loading stanford-bunny
#[allow(unused)]
pub fn load_ply<'a>(
    path: &str,
    scale: f64,
    tr: Vec3,
    color: Color,
    bxdf: Bxdf,
    medium: Medium,
    freshid: &mut FreshId,
) -> Vec<Object<'a>> {
    let (coords, indices) = parse_ply(path);
    let mut vertices = Vec::new();

    for i in 0..coords.len() / 5 {
        vertices.push(Vec3(coords[i * 5], coords[i * 5 + 1], coords[i * 5 + 2]));
    }

    let mut objs = Vec::new();

    for i in 0..indices.len() / 4 {
        let v1 = vertices[indices[i * 4 + 1]] * scale + tr;
        let v2 = vertices[indices[i * 4 + 2]] * scale + tr;
        let v3 = vertices[indices[i * 4 + 3]] * scale + tr;
        objs.push(Object::set_tri_with_medium(
            v1,
            v2,
            v3,
            bxdf,
            Texture::set_solid(color),
            freshid,
            medium,
        ));
    }

    objs
}
