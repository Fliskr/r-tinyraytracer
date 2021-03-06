extern crate nalgebra as na;

use na::Matrix;
use na::{Vector3,Vector2};
use png::HasParameters;
use regex::Regex;
use std::f64::consts::PI;
use std::f64::MAX;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::vec::Vec;


macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! multiply_vector {
    ($x:expr, $y:expr,$count: expr) => ({
        let mut result = 0.0;
        for item in 0..$count {
            result += $x[item] * $y[item];
        }
        result
    });
}

pub type Vec3 = Vector3<f64>;

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
    let fov: f64 = PI / 2.0;
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let ivory = Material {
        albedo: Vec3::new(0.6,0.3,0.1),
        diffuse_color: Vec3::new(0.4, 0.4, 0.3),
        specular_exponent: 50.0
    };
    let red_rubber = Material {
        albedo: Vec3::new(0.9,0.1,0.0),
        diffuse_color: Vec3::new(0.3, 0.1, 0.1),
        specular_exponent: 50.0
    };

    let mirror = Material {
        albedo: Vec3::new(0.9,0.1,0.8),
        diffuse_color: Vec3::new(1.0, 1.0, 1.0),
        specular_exponent: 1425.0
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
            material: mirror,
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
    let lights:Lights = vec![
        Light{
            position: Vec3::new(-20.0,20.0,120.0),
            intensity: 1.5
        }
    ];

    let mut vec: Vec<u8> = Vec::new();
    for j in 0..height {
        for i in 0..width {
            let height = f64::from(height);
            let width = f64::from(width);
            let i = f64::from(i);
            let j = f64::from(j);
            let dir_x: f64 = (i + 0.5) - width / 2.0;
            let dir_y: f64 = -(j + 0.5) + height / 2.0;
            let dir_z: f64 = -height / (2.0 * (fov / 2.0).tan());
            let dir: Vec3 = Matrix::normalize(&Vec3::new(dir_x, dir_y, dir_z));

            let ray = cast_ray(Vec3::new(0.0, 0.0, 0.0), dir, spheres.to_vec(),lights.to_vec());
            let endf: &[f64] = ray.as_slice();
            let end = &endf[..];
            let mut vec_part = vec![
                (end[0] * 255.0) as u8,
                (end[1] * 255.0) as u8,
                (end[2] * 255.0) as u8,
                255_u8,
            ];
            vec.append(&mut vec_part);
        }
    }
    writer.write_image_data(&vec.as_slice()).unwrap(); // Save
}

pub type Spheres = Vec<Sphere>;
pub type Lights = Vec<Light>;
#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
}

#[derive(Clone, Copy)]
pub struct Material {
    pub diffuse_color: Vec3,
    pub albedo: Vec3,
    pub specular_exponent: f64,
}

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f64
}

impl Sphere {
    pub fn ray_intersect(&self, orig: Vec3, dir: Vec3, t0: &mut f64) -> bool {
        let l: Vec3 = self.center - orig;
        let tca:f64 = multiply_vector!(l, dir, 3);
        let d2 = multiply_vector!(l, l, 3) - tca * tca;
        let r2 = self.radius * self.radius;
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


pub fn cast_ray(orig: Vec3, dir: Vec3, mut spheres: Spheres, lights: Lights, depth: usize = 0) -> Vec3 {
    let mut point= Vec3::new(0.0,0.0,0.0);
    let mut n = Vec3::new(0.0,0.0,0.0);
    let mut material = Material{albedo: Vec3::new(0.0,0.0,0.0),diffuse_color: Vec3::new(0.0,0.0,0.0),specular_exponent: 0.0};
    if depth>4 || !scene_intersect(orig, dir, &mut spheres, &mut point, &mut n, &mut material) {
        return Vec3::new(0.2, 0.7, 0.8);
    }

    let reflect_dir: Vec3 = Matrix::normalize(reflect(dir,n));
    let mut reflect_orig: Vec3;
    if multiply_vector!(reflect_dir, n ,3) < 0.0 {
        reflect_orig = point - n*1e-3;
    } else{
        reflect_orig = point + n* 1e-3;
    }
    let reflect_color = cast_ray(reflect_orig, reflect_dir, &mut spheres, lights, depth +1 );

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    for light in lights {
        let light_dir : Vec3 = Matrix::normalize(&(light.position - point));
        let light_distance: f64 = (light.position - point).norm();

        let mut shadow_orig: Vec3;
        if multiply_vector!(light_dir, n,3) < 0.0 {
            shadow_orig = point - n*1e-3;
            } else { 
            shadow_orig = point + n*1e-3};
        let mut shadow_pt = Vec3::new(0.0,0.0,0.0);;
        let mut shadow_n = Vec3::new(0.0,0.0,0.0);
        let mut tmpmaterial  = Material{albedo: Vec3::new(0.0,0.0,0.0),diffuse_color: Vec3::new(0.0,0.0,0.0),specular_exponent: 0.0};
        if scene_intersect(shadow_orig, light_dir, &mut spheres, &mut shadow_pt, &mut shadow_n, &mut tmpmaterial) && (shadow_pt - shadow_orig).norm() < light_distance {
            continue;
        }

        diffuse_light_intensity += light.intensity * max!(0.0_f64, multiply_vector!(light_dir, n, 3));
        specular_light_intensity += (max!(0.0, -multiply_vector!(reflect(-light_dir, n),dir,3_usize))).powf(material.specular_exponent)*light.intensity;
    }
    material.diffuse_color*diffuse_light_intensity * material.albedo[0] + Vec3::new(1.0,1.0,1.0)*specular_light_intensity *material.albedo[1] + reflect_color*material.albedo[2]
}

pub fn scene_intersect(
    orig: Vec3,
    dir: Vec3,
    spheres: &mut Spheres,
    hit: &mut Vec3,
    n: &mut  Vec3,
    material: &mut Material,
) -> bool {
    let mut spheres_dist = MAX;
    for sphere in spheres {
        let mut dist_i: f64 = 0.0;
        if sphere.ray_intersect(orig, dir, &mut dist_i) && dist_i < spheres_dist {
            spheres_dist = dist_i;
            *hit = orig + dir * dist_i;
            *n = Matrix::normalize(&(*hit - sphere.center));
            *material = sphere.material;
        }
    }
    return spheres_dist < 1000.0;
}

pub fn reflect (i: Vec3, n:Vec3) -> Vec3 {
    let z = n * 2.0;
    let x =  multiply_vector!(i,n,3_usize);
    i - z * x
} 