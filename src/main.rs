use crate::polygon::load_obj;
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
        Texture::set_checker(10, Vec3::new(0.1), Vec3::new(0.99)),
        obj_id,
    );
    let ceil = Object::set_rect(
        Axis::Y(true),
        Vec3(-25., 50., 0.),
        Vec3(25., 50., -50.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(0.99)),
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

    let light = Object::set_rect(
        Axis::Y(false),
        Vec3(-5., 49.5, -20.),
        Vec3(5., 49.5, -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3::new(60.)),
        obj_id,
    );

    let s1 = Object::set_sphere(
        Vec3(10., 7., -15.),
        7.,
        Bxdf::MicroBrdf { ax: 0.1, ay: 0.5 },
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let medium = Object::set_rect_with_medium(
        Axis::Z(true),
        Vec3(-25., 0., 0.1),
        Vec3(25., 50., 0.1),
        Bxdf::NoSurface,
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
        Medium::new(Vec3(0.004, 0.005, 0.006), Vec3(0.006, 0.002, 0.0002), 0.7),
    );

    let mut objects = vec![floor, ceil, left, right, back, light, s1, medium];

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

    let scene = Scene::new(&mut objects, Texture::set_solid(Vec3::zero()));

    let _ = render(&camera, &scene);
}

#[allow(unused)]
fn spheres() {
    let obj_id = &mut FreshId::new();

    let floor = Object::set_rect(
        Axis::Y(true),
        Vec3(-25., 10., -10.),
        Vec3(25., 10., -30.),
        Bxdf::Lambertian,
        Texture::set_solid(Vec3::new(0.99)),
        obj_id,
    );

    let light = Object::set_rect(
        Axis::Y(false),
        Vec3(-15., 40., -10.),
        Vec3(15., 40., -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3::new(10.)),
        obj_id,
    );

    let basecolor = Vec3(1., 0.1, 0.1);
    let specular = 0.7;
    let metalic = 0.5;
    let roughness = 0.1;

    let s1 = Object::set_sphere(
        Vec3(-20., 15., -20.),
        5.,
        Bxdf::set_comp(basecolor, metalic, specular, roughness),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let s2 = Object::set_sphere(
        Vec3(-7., 15., -20.),
        5.,
        Bxdf::set_comp(basecolor, metalic, specular, roughness + 0.2),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let s3 = Object::set_sphere(
        Vec3(7., 15., -20.),
        5.,
        Bxdf::set_comp(basecolor, metalic, specular, roughness + 0.4),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let s4 = Object::set_sphere(
        Vec3(20., 15., -20.),
        5.,
        Bxdf::set_comp(basecolor, metalic, specular, roughness + 0.6),
        Texture::set_solid(Vec3::new(1.)),
        obj_id,
    );

    let mut objects = vec![floor, light, s1, s2, s3, s4];

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

    //let (data, width, height) = load_hdr("assets/kloofendal_48d_partly_cloudy_puresky_1k.hdr");
    let scene = Scene::new(&mut objects, Texture::set_solid(Vec3::zero()));

    let _ = render(&camera, &scene);
}

#[allow(unused)]
fn obj() {
    let obj_id = &mut FreshId::new();
    let mut objects = load_obj("assets/voxels.obj", 10., obj_id);

    let light = Object::set_rect(
        Axis::Y(false),
        Vec3(-15., 50., -10.),
        Vec3(15., 50., -30.),
        Bxdf::Light,
        Texture::set_solid(Vec3::new(10.)),
        obj_id,
    );

    objects.push(light);

    let camera = PinholeModel::new(
        Vec3(0., 10., 30.),
        600,
        400,
        40.,
        Vec3(0., 0., -1.).normalize(),
        30.,
        2,
        2,
    );

    let scene = Scene::new(&mut objects, Texture::set_solid(Vec3::new(0.3)));

    let _ = render(&camera, &scene);
}

fn main() {
    let start = std::time::Instant::now();
    cornel_box();
    //spheres();
    //obj();
    let end = start.elapsed();
    println!("{}.{:03}sec", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
