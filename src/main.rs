#[allow(unused)]
use crate::{
    camera::{LensModel, PinholeModel},
    material::{bxdf::Bxdf, medium::Medium},
    math::Vec3,
    object::{Axis, Object},
    random::FreshId,
    render::render,
    scene::Scene,
    texture::{Texture, load_hdr},
};

#[allow(unused)]
use crate::{
    math::fmax,
    polygon::{obj_handler::load_obj, ply_handler::load_ply},
    random::XorRand,
};

mod aabb;
mod bvh;
mod camera;
mod material;
mod math;
mod object;
mod pathtracing;
mod polygon;
mod random;
mod ray;
mod render;
mod scene;
mod texture;

#[allow(unused)]
pub fn cornel_box() {
    let obj_id = &mut FreshId::new();

    let floor = Object::set_rect(
        Axis::Y(true),
        Vec3(-25., 0., 0.),
        Vec3(25., 0., -50.),
        Bxdf::Lambertian,
        Texture::set_checker(10, Vec3::new(0.1), Vec3::new(1.)),
        obj_id,
    );
    let ceil = Object::set_rect(
        Axis::Y(true),
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );
    let left = Object::set_rect(
        Axis::X(true),
        Vec3(-25., 0., 0.),
        Vec3(-25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(1., 0.1, 0.1)),
        obj_id,
    );
    let right = Object::set_rect(
        Axis::X(true),
        Vec3(25., 0., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3(0.1, 1., 0.1)),
        obj_id,
    );
    let back = Object::set_rect(
        Axis::Z(true),
        Vec3(-25., 0., -50.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(0.1)),
        obj_id,
    );

    let l1 = Object::set_rect(
        Axis::Y(false),
        Vec3(-5., 49., -20.),
        Vec3(5., 49., -30.),
        Bxdf::Light(Vec3(50., 50., 40.)),
        Texture::set_solid(Vec3::zero()),
        obj_id,
    );

    let s1 = Object::set_sphere(
        Vec3(11., 7., -15.),
        7.,
        Bxdf::MicroBrdf { ax: 0.1, ay: 0.5 },
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let water = Object::set_rect_with_medium(
        Axis::Z(true),
        Vec3(-25., 0., 0.1),
        Vec3(25., 50., 0.1),
        Bxdf::IdealGlass { ior: 1.334 },
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
        Medium::new(Vec3(0.004, 0.005, 0.006), Vec3(0.006, 0.003, 0.0002), 0.5),
    );

    let mut objects = vec![floor, ceil, left, right, back, l1, s1, water];
    objects.append(&mut load_obj("assets/cuboid.obj", 1., obj_id));

    let rand = &mut XorRand::new(25);
    for _ in 0..20 {
        let y = rand.next01() * 40. + 5.;
        let x = rand.next01() * fmax(y / 4., 2.) - 20.;
        let z = -rand.next01() * fmax(y / 4., 2.) - 5.;
        let r = rand.next01() * fmax(y / 15., 1.);
        let bubble = Object::set_sphere(
            Vec3(x, y, z),
            r,
            Bxdf::IdealGlass { ior: 1. },
            Texture::set_solid(Vec3::new(1.)),
            obj_id,
        );
        objects.push(bubble);
    }

    let camera = PinholeModel::new(
        Vec3(0., 25., 75.),
        600,
        600,
        30.,
        Vec3(0., 0., -1.).normalize(),
        40.,
        3,
        3,
    );

    let scene = Scene::new(&mut objects, Texture::set_solid(Vec3::zero()));

    let _ = render(&camera, &scene);
}

pub fn material_test() {
    let obj_id = &mut FreshId::new();

    let floor = Object::set_rect(
        Axis::Y(true),
        Vec3(-200., 10., 200.),
        Vec3(200., 10., -200.),
        Bxdf::Lambertian,
        Texture::set_checker(40, Vec3::new(0.1), Vec3::new(0.8)),
        obj_id,
    );

    let light = Object::set_rect(
        Axis::Y(false),
        Vec3(-15., 50., -5.),
        Vec3(15., 50., -35.),
        Bxdf::Light(Vec3::new(3.)),
        Texture::set_solid(Vec3::new(3.)),
        obj_id,
    );

    let s1 = Object::set_sphere_with_medium(
        Vec3(-1., 17., -10.),
        7.,
        Bxdf::IdealGlass { ior: 1.5 },
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
        Medium::new(Vec3::new(0.2), Vec3(0.2, 0.1, 0.2), 0.7),
    );

    let s2 = Object::set_sphere(
        Vec3(38., 17., -10.),
        7.,
        Bxdf::Lambertian,
        Texture::Gradation,
        obj_id,
    );

    let mut objects = vec![floor, light, s1, s2];
    objects.append(&mut load_ply(
        "assets/bunny.ply",
        120.,
        Vec3(20., 7., -11.),
        Vec3::new(1.),
        Bxdf::set_comp(Vec3(1., 0.1, 0.1), 0.5, 0.7, 0.2),
        Medium::no_medium(),
        obj_id,
    ));

    let camera = LensModel::new(
        800,
        450,
        Vec3(0.27, -0.5, -1.).normalize(),
        Vec3(3., 41., 32.),
        30.,
        2.,
        15.,
        29.,
        100.,
        14,
        14,
    );

    let scene = Scene::new(&mut objects, Texture::set_solid(Vec3::zero()));

    let _ = render(&camera, &scene);
}

fn main() {
    //cornel_box();
    material_test();
}
