use crate::{
    camera::{LensModel, PinholeModel},
    material::bxdf::Bxdf,
    math::Vec3,
    object::{Axis, Object},
    random::FreshId,
    render::render,
    scene::Scene,
};

mod aabb;
mod bvh;
mod camera;
mod material;
mod math;
mod object;
mod pathtracing;
mod random;
mod ray;
mod render;
mod scene;

pub fn cornel_box() {
    let obj_id = &mut FreshId::new();

    let rect0 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Vec3::new(0.99),
        obj_id,
    );
    let rect1 = Object::set_rect(
        Axis::Y,
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3::new(0.99),
        obj_id,
    );
    let rect2 = Object::set_rect(
        Axis::X,
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(1., 0.1, 0.1),
        obj_id,
    );
    let rect3 = Object::set_rect(
        Axis::X,
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3(0.1, 1., 0.1),
        obj_id,
    );
    let rect4 = Object::set_rect(
        Axis::Z,
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Vec3::new(0.99),
        obj_id,
    );

    let rect5 = Object::set_rect(
        Axis::Y,
        Vec3(-5., 49.99, -20.),
        Vec3(5., 49.99, -30.),
        Bxdf::Light,
        Vec3::new(45.),
        obj_id,
    );

    let sphere = Object::set_sphere(
        Vec3(0., 7.5, -25.),
        7.5,
        Bxdf::CompositeBrdf {
            basecolor: Vec3::new(1.),
            metalic: 0.2,
            highlight: Vec3::new(1.),
            roughness: 0.2,
        },
        Vec3::new(1.),
        obj_id,
    );

    let objects = vec![&rect0, &rect1, &rect2, &rect3, &rect4, &rect5, &sphere];

    let camera = LensModel::new(
        600,
        600,
        Vec3(0., 0., -1.).normalize(),
        Vec3(0., 25., 120.),
        30.,
        2.,
        42.,
        96.,
        100.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::zero());

    let _ = render(&camera, &scene);
}

fn spheres() {
    let obj_id = &mut FreshId::new();
    let floor = Object::set_rect(
        Axis::Y,
        Vec3(-25., 10., -10.),
        Vec3(25., 10., -30.),
        Bxdf::Lambertian,
        Vec3::new(0.99),
        obj_id,
    );

    let light = Object::set_rect(
        Axis::Y,
        Vec3(-15., 40., -10.),
        Vec3(15., 40., -30.),
        Bxdf::Light,
        Vec3::new(10.),
        obj_id,
    );

    let specular = 0.7;
    let metalic = 0.5;
    let roughness = 0.1;

    let s1 = Object::set_sphere(
        Vec3(-20., 15., -20.),
        5.,
        Bxdf::set_comp(Vec3(1., 0.1, 0.1), metalic, specular, roughness),
        Vec3(1., 0.1, 0.1),
        obj_id,
    );

    let s2 = Object::set_sphere(
        Vec3(-7., 15., -20.),
        5.,
        Bxdf::set_comp(Vec3(1., 0.1, 0.1), metalic, specular, roughness + 0.2),
        Vec3(1., 0.1, 0.1),
        obj_id,
    );

    let s3 = Object::set_sphere(
        Vec3(7., 15., -20.),
        5.,
        Bxdf::set_comp(Vec3(1., 0.1, 0.1), metalic, specular, roughness + 0.4),
        Vec3(1., 0.1, 0.1),
        obj_id,
    );

    let s4 = Object::set_sphere(
        Vec3(20., 15., -20.),
        5.,
        Bxdf::set_comp(Vec3(1., 0.1, 0.1), metalic, specular, roughness + 0.6),
        Vec3::new(1.),
        obj_id,
    );

    let objects = vec![&floor, &light, &s1, &s2, &s3, &s4];

    let camera = PinholeModel::new(
        Vec3(0., 25., 30.),
        600,
        400,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        4,
        4,
    );

    let scene = Scene::new(objects, Vec3::zero());

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    //cornel_box();
    spheres();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
