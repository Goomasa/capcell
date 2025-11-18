use crate::{
    math::{PI, Point3, Vec3, cross, dot},
    random::XorRand,
};

pub trait Camara: Send + Sync {
    fn get_pixel(&self) -> (u32, u32);
    fn get_sample(&self) -> (u32, u32);
    fn get_coeff(&self) -> f64;
    fn setup(&self, u: u32, v: u32, su: u32, sv: u32, rand: &mut XorRand) -> (f64, Vec3, Vec3);
    //return (g_term, org, dir)
}

pub struct LensModel {
    pixel_w: u32,
    pixel_h: u32,
    sensor_dir: Vec3,
    sensor_w: f64,
    sensor_h: f64,
    sensor_u: Vec3,
    sensor_v: Vec3,
    sensor_corner: Point3,
    superpx_u: Vec3,
    superpx_v: Vec3,
    sensor_to_lens: f64,
    lens_radius: f64,
    lens_center: Point3,
    lens_to_plane: f64,
    sensitivity: f64,
    spp: u32,  //samples per pixel
    sspp: u32, //super samples per pixel
}

#[allow(unused)]
impl LensModel {
    pub fn new(
        px_w: u32,
        px_h: u32,
        sensor_dir: Vec3,
        sensor_center: Point3,
        sensor_w: f64,
        lens_r: f64,
        focal_len: f64, // =sensor_to_lens
        lens_to_plane: f64,
        sensitivity_scale: f64,
        spp: u32,
        sspp: u32,
    ) -> Self {
        let sensor_h = sensor_w * (px_h as f64 / px_w as f64);
        let w_per_px = sensor_w / px_w as f64;
        let h_per_px = sensor_h / px_h as f64;
        let sensitivity = sensitivity_scale / (w_per_px * h_per_px);

        let sensor_u = cross(sensor_dir, Vec3(0., 1., 0.)).normalize();
        let sensor_v = cross(sensor_dir, sensor_u).normalize();
        let sensor_corner = sensor_center - sensor_u * sensor_w / 2. - sensor_v * sensor_h / 2.;
        let superpx_u = sensor_u * w_per_px / sspp as f64;
        let superpx_v = sensor_v * h_per_px / sspp as f64;
        let lens_center = sensor_center + sensor_dir * focal_len;

        LensModel {
            pixel_w: px_w,
            pixel_h: px_h,
            sensor_dir: sensor_dir,
            sensor_w: sensor_w,
            sensor_h: sensor_h,
            sensor_u: sensor_u,
            sensor_v: sensor_v,
            sensor_corner: sensor_corner,
            superpx_u: superpx_u,
            superpx_v: superpx_v,
            sensor_to_lens: focal_len,
            lens_radius: lens_r,
            lens_center: lens_center,
            lens_to_plane: lens_to_plane,
            sensitivity,
            spp: spp,
            sspp: sspp,
        }
    }

    fn sample_lens(&self, pixel_pos: Point3, rand: &mut XorRand) -> (f64, Point3) {
        //return (coefficient=cos^2/l^2, sample_pos)
        let theta = 2.0 * PI * rand.next01();
        let r = rand.next01().sqrt() * self.lens_radius;

        let lens_pos =
            self.lens_center + self.sensor_u * r * theta.cos() + self.sensor_v * r * theta.sin();
        let l_sq = (lens_pos - pixel_pos).length_sq();
        let cos_theta = dot((lens_pos - pixel_pos).normalize(), self.sensor_dir);
        (cos_theta * cos_theta / l_sq, lens_pos)
    }

    fn first_dir(&self, pixel_pos: Point3, lens_pos: Point3) -> Vec3 {
        let plane_pos = (self.lens_center - pixel_pos) * (self.sensor_to_lens + self.lens_to_plane)
            / self.sensor_to_lens
            + pixel_pos;
        (plane_pos - lens_pos).normalize()
    }
}

impl Camara for LensModel {
    fn get_pixel(&self) -> (u32, u32) {
        (self.pixel_w, self.pixel_h)
    }

    fn get_sample(&self) -> (u32, u32) {
        (self.spp, self.sspp)
    }

    fn get_coeff(&self) -> f64 {
        let w_per_px = self.sensor_w / self.pixel_w as f64;
        let h_per_px = self.sensor_h / self.pixel_h as f64;
        let lens_area = PI * self.lens_radius * self.lens_radius;
        self.sensitivity * w_per_px * h_per_px * lens_area
            / (self.spp as f64 * self.sspp.pow(2) as f64)
    }

    fn setup(&self, u: u32, v: u32, su: u32, sv: u32, rand: &mut XorRand) -> (f64, Vec3, Vec3) {
        let u = self.pixel_w - u - 1;
        let v = self.pixel_h - v - 1;
        let pixel_pos = self.sensor_corner
            + self.superpx_u * ((u * self.sspp + su) as f64 + 0.5)
            + self.superpx_v * ((v * self.sspp + sv) as f64 + 0.5);

        let (g_term, lens_pos) = self.sample_lens(pixel_pos, rand);
        let dir = self.first_dir(pixel_pos, lens_pos);

        (g_term, lens_pos, dir)
    }
}

pub struct PinholeModel {
    eye: Point3,
    pixel_w: u32,
    pixel_h: u32,
    sensor_corner: Point3,
    superpx_u: Vec3,
    superpx_v: Vec3,
    spp: u32,
    sspp: u32,
}

#[allow(unused)]
impl PinholeModel {
    pub fn new(
        eye_pos: Point3,
        px_w: u32,
        px_h: u32,
        sensor_w: f64,
        eye_dir: Vec3,
        eye_to_sensor: f64,
        spp: u32,
        sspp: u32,
    ) -> Self {
        let sensor_h = sensor_w * (px_h as f64 / px_w as f64);
        let sensor_u = cross(eye_dir, Vec3(0., 1., 0.)).normalize();
        let sensor_v = cross(eye_dir, sensor_u).normalize();
        let superpx_u = sensor_u * sensor_w / px_w as f64 / sspp as f64;
        let superpx_v = sensor_v * sensor_h / px_h as f64 / sspp as f64;
        let sensor_corner =
            eye_pos + eye_dir * eye_to_sensor - sensor_u * sensor_w / 2. - sensor_v * sensor_h / 2.;

        PinholeModel {
            eye: eye_pos,
            pixel_w: px_w,
            pixel_h: px_h,
            sensor_corner: sensor_corner,
            superpx_u: superpx_u,
            superpx_v: superpx_v,
            spp: spp,
            sspp: sspp,
        }
    }
}

impl Camara for PinholeModel {
    fn get_pixel(&self) -> (u32, u32) {
        (self.pixel_w, self.pixel_h)
    }

    fn get_sample(&self) -> (u32, u32) {
        (self.spp, self.sspp)
    }

    fn get_coeff(&self) -> f64 {
        1. / (self.spp * self.sspp * self.sspp) as f64
    }

    fn setup(&self, u: u32, v: u32, su: u32, sv: u32, _: &mut XorRand) -> (f64, Vec3, Vec3) {
        let pixel_pos = self.sensor_corner
            + self.superpx_u * ((u * self.sspp + su) as f64 + 0.5)
            + self.superpx_v * ((v * self.sspp + sv) as f64 + 0.5);
        let dir = (pixel_pos - self.eye).normalize();

        (1.0, self.eye, dir)
    }
}
