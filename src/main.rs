extern crate nalgebra as na;

use na::{Vector3, Vector4};
use png::HasParameters;
use regex::Regex;
use std::f32::consts::PI;
use std::f32::MAX;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::mem::swap;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::vec::Vec;

pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
fn main() {
    let paths = fs::read_dir("./images/").unwrap();
    let re = Regex::new(r"(\d+).png").unwrap();
    let mut index = 0;
    for path in paths {
        let path_name = format!("{}", &path.unwrap().path().display());
        if re.is_match(&path_name) {
            let caps = re.captures(&path_name).unwrap();
            if caps.get(1).unwrap().as_str().parse::<i32>().unwrap() > index {
                index = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
            }
        }
    }
    index += 1;
    let path_name = format!("./images/image{}.png", index);
    let path = Path::new(&path_name);
    let file = File::create(path).unwrap();

    let ref mut w = BufWriter::new(file);
    let width = 1024;
    let height = 768;
    // let width = 4;
    // let height = 4;
    let fov: f32 = PI / 2.0;
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let ivory = Material {
        refractive_index: 1.0,
        albedo: Vec4::new(0.6, 0.3, 0.1, 0.0),
        diffuse_color: Vec3::new(0.4, 0.4, 0.3),
        specular_exponent: 50.0,
    };
    let glass = Material {
        refractive_index: 1.5,
        albedo: Vec4::new(0.0, 0.5, 0.1, 0.8),
        diffuse_color: Vec3::new(0.6, 0.7, 0.8),
        specular_exponent: 125.0,
    };
    let red_rubber = Material {
        refractive_index: 1.0,
        albedo: Vec4::new(0.9, 0.1, 0.0, 0.0),
        diffuse_color: Vec3::new(0.3, 0.1, 0.1),
        specular_exponent: 10.0,
    };

    let mirror = Material {
        refractive_index: 1.0,
        albedo: Vec4::new(0.0, 10.0, 0.8, 0.0),
        diffuse_color: Vec3::new(1.0, 1.0, 1.0),
        specular_exponent: 1425.0,
    };

    let spheres: Spheres = vec![
        Sphere {
            center: Vec3::new(-3.0, 0.0, -16.0),
            radius: 2.0,
            material: ivory,
        },
        Sphere {
            center: Vec3::new(-1.0, -1.5, -12.0),
            radius: 2.0,
            material: glass,
        },
        Sphere {
            center: Vec3::new(1.5, -0.5, -18.0),
            radius: 3.0,
            material: red_rubber,
        },
        Sphere {
            center: Vec3::new(7.0, 5.0, -18.0),
            radius: 4.0,
            material: mirror,
        },
    ];
    let lights: Lights = vec![
        Light {
            position: Vec3::new(-20.0, 20.0, 20.0),
            intensity: 1.5,
        },
        Light {
            position: Vec3::new(30.0, 50.0, -25.0),
            intensity: 1.8,
        },
        Light {
            position: Vec3::new(30.0, 20.0, 30.0),
            intensity: 1.7,
        },
    ];

    let mut vec: Vec<u8> = Vec::new();
    let (tx, rx) = mpsc::channel();
    let threads = 4;
    let mut vecs: Vec<Vec<u8>> = vec![vec![];threads];
    for k in 0..threads {
        let rest_height = height / threads;
        let start = k + rest_height * k;
        let end = k + rest_height * (k + 1);
        let tx1 = mpsc::Sender::clone(&tx);
        let spheres = spheres.clone();
        let lights = lights.clone();
        thread::spawn(move || {
            println!("thread:{}", k);
            for j in start..end {
                for i in 0..width {
                    let height: f32 = height as f32;
                    let width: f32 = width as f32;
                    let i = i as f32;
                    let j = j as f32;
                    // let dir_x: f32 = (i + 0.5) - width / 2.0;
                    // let dir_y: f32 = -(j + 0.5) + height / 2.0;
                    // let dir_z: f32 = -height / (2.0 * (fov / 2.0).tan());
                    let x: f32 =
                        (2.0 * (i + 0.5) / width - 1.0) * (fov / 2.0).tan() * width / height;
                    let y: f32 = -(2.0 * (j + 0.5) / height - 1.0) * (fov / 2.0).tan();
                    let dir: Vec3 = Vec3::new(x, y, -1.0).normalize();
                    // let dir: Vec3 = Vec3::new(dir_x, dir_y, dir_z).normalize();

                    let ray = cast_ray(
                        Vec3::new(0.0, 0.0, 0.0),
                        dir,
                        spheres.clone(),
                        lights.clone(),
                        0,
                    );
                    let val = (
                        (1f32.min(ray.x) * 255.0) as u8,
                        (1f32.min(ray.y) * 255.0) as u8,
                        (1f32.min(ray.z) * 255.0) as u8,
                        k,
                    );
                    tx1.send(val).unwrap();
                }
            }
            println!("thread:{} ended", k);
        });
    }
    for _ in 0..(width * height) {
        let (a, b, c, k) = rx.recv().unwrap();
        let vec_part:Vec<u8> = vec![
            a,
            b,
            c,
            255u8,
        ];
        vecs[k].extend(vec_part.clone());
    }
    for vc in vecs {
        vec.append(&mut vc.clone());
    }

    writer.write_image_data(&vec.as_slice()).unwrap(); // Save
}

pub type Spheres = Vec<Sphere>;
pub type Lights = Vec<Light>;
#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

#[derive(Clone, Copy)]
pub struct Material {
    pub refractive_index: f32,
    pub diffuse_color: Vec3,
    pub albedo: Vec4,
    pub specular_exponent: f32,
}

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f32,
}

impl Sphere {
    pub fn ray_intersect(&self, orig: Vec3, dir: Vec3, t0: &mut f32) -> bool {
        let l: Vec3 = self.center - orig;
        let tca: f32 = l.dot(&dir);
        let d2 = l.dot(&l) - tca.powf(2.0); // * tca;
        let r2 = self.radius.powf(2.0); // * self.radius;
        if d2 > r2 {
            return false;
        }
        let thc = (r2 - d2).sqrt();
        *t0 = tca - thc;
        let t1 = tca + thc;
        if *t0 < 0.0 {
            *t0 = t1;
        }
        if *t0 < 0.0 {
            return false;
        }
        return true;
    }
}

pub fn cast_ray(orig: Vec3, dir: Vec3, spheres: Spheres, lights: Lights, depth: usize) -> Vec3 {
    let mut point = Vec3::new(0.0, 0.0, 0.0);
    let mut n = Vec3::new(0.0, 0.0, 0.0);
    let mut material = Material {
        refractive_index: 0.0,
        albedo: Vec4::new(0.0, 0.0, 0.0, 0.0),
        diffuse_color: Vec3::new(0.0, 0.0, 0.0),
        specular_exponent: 0.0,
    };
    if depth > 4
        || !scene_intersect(
            orig.clone(),
            dir.clone(),
            spheres.clone(),
            &mut point,
            &mut n,
            &mut material,
        )
    {
        return Vec3::new(0.2, 0.7, 0.8);
    }

    let reflect_dir: Vec3 = reflect(&dir, &n).normalize();
    let refract_dir: Vec3 = refract(&dir, &n, material.refractive_index).normalize();
    let reflect_orig: Vec3;
    if reflect_dir.dot(&n) < 0.0 {
        reflect_orig = point - n * 1e-2;
    } else {
        reflect_orig = point + n * 1e-2;
    }
    let refract_orig: Vec3;
    if refract_dir.dot(&n) < 0.0 {
        refract_orig = point - n * 1e-2;
    } else {
        refract_orig = point + n * 1e-2;
    }
    let reflect_color = cast_ray(
        reflect_orig,
        reflect_dir,
        spheres.clone(),
        lights.clone(),
        depth + 1,
    );
    let refract_color = cast_ray(
        refract_orig,
        reflect_dir,
        spheres.clone(),
        lights.clone(),
        depth + 1,
    );

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    for light in lights {
        let light_dir: Vec3 = (light.position - point).normalize();
        let light_distance: f32 = (light.position - point).norm();

        let shadow_orig: Vec3;
        if light_dir.dot(&n) < 0.0 {
            shadow_orig = point - n * 1e-2;
        } else {
            shadow_orig = point + n * 1e-2
        };
        let mut shadow_pt = Vec3::new(0.0, 0.0, 0.0);
        let mut shadow_n = Vec3::new(0.0, 0.0, 0.0);
        let mut tmpmaterial = Material {
            refractive_index: 0.0,
            albedo: Vec4::new(0.0, 0.0, 0.0, 0.0),
            diffuse_color: Vec3::new(0.0, 0.0, 0.0),
            specular_exponent: 0.0,
        };
        if scene_intersect(
            shadow_orig.clone(),
            light_dir.clone(),
            spheres.clone(),
            &mut shadow_pt,
            &mut shadow_n,
            &mut tmpmaterial,
        ) && (shadow_pt - shadow_orig).norm() < light_distance
        {
            continue;
        }

        diffuse_light_intensity += light.intensity * 0f32.max(light_dir.dot(&n));
        specular_light_intensity += 0f32
            .max(-reflect(&(-light_dir), &n).dot(&dir))
            .powf(material.specular_exponent)
            * light.intensity;
    }
    material.diffuse_color * diffuse_light_intensity * material.albedo[0]
        + Vec3::new(1.0, 1.0, 1.0) * specular_light_intensity * material.albedo[1]
        + reflect_color * material.albedo[2]
        + refract_color * material.albedo[3]
}

pub fn scene_intersect(
    orig: Vec3,
    dir: Vec3,
    spheres: Spheres,
    hit: &mut Vec3,
    n: &mut Vec3,
    material: &mut Material,
) -> bool {
    let mut spheres_dist = MAX;
    for sphere in spheres {
        let mut dist_i: f32 = 0.0;
        if sphere.ray_intersect(orig.clone(), dir.clone(), &mut dist_i) && dist_i < spheres_dist {
            spheres_dist = dist_i;
            *hit = orig + dir * dist_i;
            *n = (*hit - sphere.center).normalize();
            *material = sphere.material;
        }
    }
    let mut checkerboard_dist = MAX;
    if dir.y.abs() > 1e-3 {
        let d = -(orig.y + 4.0) / dir.y;
        let pt = orig + dir * d;
        let z = pt.z < -10f32 && pt.z > -30f32;
        if d > 0f32 && pt.x.abs() < 10f32 && z && d < spheres_dist {
            checkerboard_dist = d;
            *hit = pt;
            *n = Vec3::new(0f32, 1f32, 0f32);
            if (((0.5 * hit.x + 1000.0) as i32) + ((0.5 * hit.z) as i32) & 1) == 1 {
                material.diffuse_color = Vec3::new(0.3, 0.3, 0.3);
            } else {
                material.diffuse_color = Vec3::new(0.3, 0.8 * 0.3, 0.3 * 0.3);
            }
            // *material.diffuse_color = material.diffuse_color * 0.3;
        }
    }
    spheres_dist.min(checkerboard_dist) < 1000.0
}

pub fn reflect(i: &Vec3, n: &Vec3) -> Vec3 {
    i - n * 2.0 * i.dot(&n)
}

pub fn refract(i: &Vec3, n: &Vec3, refractive_index: f32) -> Vec3 {
    let mut cosi = -(-1f32.max(1f32.min(i.dot(&n))));
    let mut etai = 1f32;
    let mut etat = refractive_index;
    let mut nn = n.clone();
    if cosi < 0f32 {
        cosi = -cosi;
        swap(&mut etai, &mut etat);
        nn = -n.clone();
    }
    let eta = etai / etat;
    let k = 1f32 - eta * eta * (1f32 - cosi * cosi);
    if k < 0f32 {
        Vec3::new(0.0, 0.0, 0.0)
    } else {
        i * eta + nn * (eta * cosi - k.sqrt())
    }
}
