extern crate image;
extern crate nalgebra;

use image::{ImageBuffer, Rgb, Pixel};

use nalgebra::{cross, dot, Norm};

type Vec3 = nalgebra::Vec3<f32>;

const OUT_FILE: &'static str = "image.png";
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

struct Ray {
    origin: Vec3,
    dir: Vec3,
}

impl Ray {
    fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin: origin, dir: dir.normalize() }
    }
}

#[derive(Clone, Debug)]
struct Intersection {
    pos: Vec3,
    normal: Vec3,
    dist: f32,
}

impl Intersection {
    fn new(pos: Vec3, normal: Vec3, dist: f32) -> Self {
        Intersection { pos: pos, normal: normal, dist: dist }
    }
}

trait Surface {
    fn intersect(&self, &Ray) -> Option<Intersection>;
}

trait Material {
    fn color(&self, ray: &Ray, &Intersection) -> (u8, u8, u8);
}

#[derive(Clone, Debug)]
struct Sphere {
    pos: Vec3,
    radius: f32,
}

impl Surface for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let center_offset = ray.origin - self.pos;
        let b = 2. * dot(&ray.dir, &center_offset);
        let c = center_offset.sqnorm() - self.radius * self.radius;

        let discriminant = b * b - 4. * c;

        if discriminant >= 0. {
            //println!("{:?} {:?}", ray.origin, ray.dir);

            let disc_sqrt = discriminant.sqrt();
            let d1 = 0.5 * (-b + disc_sqrt);
            let d2 = 0.5 * (-b - disc_sqrt);

            // d1 should always be larger than d2, we want the smallest positive distance
            let d = if d2 > 0. {
                d2
            } else if d1 > 0. {
                // We are inside the sphere
                d1
            } else {
                // Both are negative, sphere is behind camera
                return None
            };

            let pos = ray.origin + ray.dir * d;
            let normal = (pos - self.pos).normalize();
            Some(Intersection::new(pos, normal, d))
        } else {
            None
        }
    }
}

struct DiffuseMaterial {
    intensity: f32,
    color: (u8, u8, u8),
}

impl DiffuseMaterial {
    fn new(intensity: f32, color: (u8, u8, u8)) -> Self {
        DiffuseMaterial { intensity: intensity, color: color }
    }
}

impl Material for DiffuseMaterial {
    fn color(&self, ray: &Ray, intersection: &Intersection) -> (u8, u8, u8) {
        // Note: ray direction has to be flipped
        let f = f32::max(0., dot(&intersection.normal, &-ray.dir));
        let f = f * self.intensity;
        ((self.color.0 as f32 * f) as u8,
         (self.color.1 as f32 * f) as u8,
         (self.color.2 as f32 * f) as u8)
    }
}

impl Sphere {
    fn new(pos: Vec3, radius: f32) -> Self {
        Sphere { pos: pos, radius: radius }
    }
}

struct Camera {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,
    right: Vec3,
}

struct PointLight {
    intensity: f32,
}

impl PointLight {
    fn new(intensity: f32) -> Self {
        PointLight { intensity: intensity }
    }
}

struct Scene {
    objects: Vec<Sphere>,
    lights: Vec<PointLight>,
    ambient_const: f32,
    camera: Camera,
}

impl Scene {
    fn new(objects: Vec<Sphere>, lights: Vec<PointLight>, ambient: f32, camera: Camera) -> Self {
        Scene { objects: objects, lights: lights, ambient_const: ambient, camera: camera }
    }

    fn intersect(&self, ray: &Ray) -> Option<(&Sphere, Intersection)> {
        let mut result = None;
        for obj in self.objects.iter() {
            if let Some(hit) = obj.intersect(ray) {
                match result.clone() {
                    None => result = Some((obj, hit)),
                    Some((_, ref old_hit)) =>
                        if hit.dist < old_hit.dist { result = Some((obj, hit)) }
                }
            }
        }
        result
    }
}

impl Camera {
    fn new(pos: Vec3, dir: Vec3, up: Vec3) -> Self {
        let right = cross(&up, &dir).normalize();
        let up = cross(&right, &dir).normalize();
        Camera { pos: pos, dir: dir.normalize(), up: up, right: right }
    }

    fn from_lookat(pos: Vec3, lookat: Vec3, up: Vec3) -> Self {
        let dir = lookat - pos;
        Camera::new(pos, dir, up)
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        let norm_x = (x as f32 / WIDTH as f32) - 0.5;
        let norm_y = (y as f32 / HEIGHT as f32) - 0.5;

        let dir = self.right * norm_x + self.up * norm_y + self.dir;
        Ray::new(self.pos, dir)
    }
}

fn main() {
    let mut im: ImageBuffer<Rgb<u8>, _> = ImageBuffer::new(WIDTH, HEIGHT);

    let camera = {
        let pos = Vec3::new(0., 0., -4.);
        let lookat = Vec3::new(0., 0., 0.);
        let up = Vec3::new(0., 1., 0.);
        Camera::from_lookat(pos, lookat, up)
    };
    let sphere = Sphere::new(Vec3::new(0., 0., 0.), 1.);
    let material = DiffuseMaterial::new(0.7, (0, 0, 255));
    let light = PointLight::new(1.);

    let scene = Scene::new(vec![sphere], vec![light], 1., camera);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let ray = scene.camera.get_ray(x, y);
            if let Some((_, hit)) = scene.intersect(&ray) {
                let color = material.color(&ray, &hit);
                let color = Rgb::from_channels(color.0, color.1, color.2, 255);
                im.put_pixel(x, y, color);
            }
        }
    }

    im.save(OUT_FILE).unwrap();
}
