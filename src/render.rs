use bmp::{Image, Pixel, px};
//using crate "bmp", https://github.com/sondrele/rust-bmp/tree/master/src
use rayon::prelude::*;
//using crate "rayon", https://github.com/rayon-rs/rayon

use crate::{
    camera::Camara,
    math::{Color, Vec3, gamma, is_valid},
    pathtracing::Pathtracing,
    random::XorRand,
    ray::Ray,
    scene::Scene,
};

pub fn render(camera: &impl Camara, scene: &Scene) {
    let (pixel_w, pixel_h) = camera.get_pixel();
    let (spp, sspp) = camera.get_sample();
    let coeff = camera.get_coeff();

    let mut buffer = vec![Vec3::new(0.); (pixel_w * pixel_h) as usize];
    let mut img = Image::new(pixel_w, pixel_h);

    buffer
        .par_chunks_mut(pixel_w as usize)
        .enumerate()
        .for_each(|(v, row)| {
            for u in 0..pixel_w {
                let mut rand = XorRand::new(u * v as u32);
                let mut accumlated_color = Color::new(0.);

                for sv in 0..sspp {
                    for su in 0..sspp {
                        let (g_term, org, dir) = camera.setup(u, v as u32, su, sv, &mut rand);

                        for _ in 0..spp {
                            let ray = Ray { org, dir };
                            let mut tracer = Pathtracing::new(ray);
                            let rad = tracer.integrate(scene, &mut rand) * g_term;
                            if !is_valid(&rad) {
                                continue;
                            }

                            accumlated_color = accumlated_color + rad;
                        }
                    }
                }
                row[u as usize] = accumlated_color * coeff;
            }
            println!("{v}");
        });

    for i in 0..pixel_w * pixel_h {
        let y = i / pixel_w;
        let x = i - pixel_w * y;
        let rgb = gamma(buffer[i as usize]);
        img.set_pixel(x, y as u32, px!(rgb.0, rgb.1, rgb.2));
    }
    let _ = img.save("render.bmp");
}
