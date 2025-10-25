use crate::{
    camera::LensModel,
    material::Bxdf,
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

    let objects = vec![&rect0, &rect1, &rect2, &rect3, &rect4, &rect5];

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

fn main() {
    let start = std::time::Instant::now();
    cornel_box();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
