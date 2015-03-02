#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(core)]
#![feature(old_io)]
#![feature(old_path)]
#![feature(str_words)]

extern crate core;
use core::num::*;
use std::cmp;
use std::num;

extern crate vecmath;
use vecmath::{Vector2, Vector3, Matrix3x2, Matrix3, vec3_sub, vec3_cross, vec3_dot, vec3_normalized};

extern crate bmp;
use bmp::{Image, Pixel, consts};

fn main() {
    let box ref model = load("models/african_head.obj");
    let mut image = Image::new(500, 500);
    draw_poly(&mut image, model);
    let bmp = "/tmp/opengl.bmp";
    let _ = image.save(bmp).unwrap_or_else(|e| {
        panic!("Failed to save {}: {}", bmp, e)
    });
}

fn pixel(image: &mut Image, x: u32, y: u32, val: Pixel) {
    let w = image.get_width();
    let h = image.get_height();
    if x < w && y < h {
        image.set_pixel(x, h - y - 1, val)
    }
}

fn line(x0: u32, y0: u32, x1: u32, y1: u32, image: &mut Image, color: Pixel) {
    //println!("{} {} -> {} {}", x0, y0, x1, y1);
    let dx = x1 as i32 - x0 as i32;
    let dy = y1 as i32 - y0 as i32;
    // swap x/y to iterate over longest coord
    let swap = dx.abs() < dy.abs();
    let (_p0, _p1, _q0, _q1, _dp, _dq) = if !swap { (x0, x1, y0, y1, dx, dy) } else { (y0, y1, x0, x1, dy, dx) };
    // swap to iterate from smaller `p` coord value to larger
    let (p0, p1, q0, q1) = if _p0 < _p1 { (_p0, _p1, _q0, _q1) } else { (_p1, _p0, _q1, _q0) };
    let (dp, dq, q_add) = (_dp.abs(), _dq.abs(),  if q0 < q1 { 1 } else { -1 });
    let q_err_add = dq*2;
    let mut q_err = 0;
    let mut q = q0 as i32;
    for p in p0..(p1+1) {
        if !swap { pixel(image, p, q as u32, color) } else { pixel(image, q as u32, p, color) }
        q_err += q_err_add;
        if q_err > dp {
            q += q_add;
            q_err -= dp*2
        }
    }
}

#[derive(Debug)]
struct Vec3f {
    x: f32, y: f32, z: f32
}

struct Model {
    verts: Vec<Vec3f>,
    faces: Vec<Vec<u32>>
}

impl Model {
    fn new() -> Box<Model> {
        box Model{ verts: vec![], faces: vec![] }
    }
}

use std::old_io::BufferedReader;
use std::old_io::File;

fn load(filename: &str) -> Box<Model> {
    let mut model = Model::new();
    let mut file = BufferedReader::new(File::open(&Path::new(filename)));
    fn v(s: &str) -> f32 { num::from_str_radix::<f32>(s, 10).unwrap_or(0.) };
    fn f(s: &str) -> u32 { s.splitn(1, '/').next().and_then(|_1| num::from_str_radix::<u32>(_1, 10).ok()).map(|f| f-1).unwrap_or(0) };
    for maybe in file.lines() {
        match maybe {
            Err(err) => panic!("I/O error reading {}: {}", filename, err),
            Ok(line) => {
                match line.words().collect::<Vec<_>>().as_slice() {
                    ["v", x, y, z]    => model.verts.push(Vec3f{ x: v(x), y: v(y), z: v(z) }),
                    ["f", _1, _2, _3] => model.faces.push(vec![f(_1), f(_2), f(_3)]),
                    _ => () ,
                }
            }
        }
    }
    model
}

fn draw_wireframe(image: &mut Image, model: &Model) {
    let w2 = image.get_width()  as f32 / 2.;
    let h2 = image.get_height() as f32 / 2.;
    for ref face in &model.faces {
        //println!("face = {:?}", face);
        for i in 0..3 {
            let ref v0 = model.verts[face[i] as usize];
            let ref v1 = model.verts[face[if i < 2 {i+1} else {0}] as usize];
            //println!("v0 = {:?}; v1 = {:?}", v0, v1);
            let x0 = (v0.x+1.)*w2;
            let y0 = (v0.y+1.)*h2;
            let x1 = (v1.x+1.)*w2;
            let y1 = (v1.y+1.)*h2;
            line(x0 as u32, y0 as u32, x1 as u32, y1 as u32, image, consts::WHITE);
        }
    }
}

type Point = Vector2<i32>;
type Triangle = Matrix3x2<i32>;
type Triangle3f = Matrix3<f32>;
type Baricentric = Vector3<f32>;

fn barycentric(t: Triangle, p: Point) -> Baricentric {
    let u = vec3_cross(
        [(t[2][0]-t[0][0]) as f32, (t[1][0]-t[0][0]) as f32, (t[0][0]-p[0]) as f32],
        [(t[2][1]-t[0][1]) as f32, (t[1][1]-t[0][1]) as f32, (t[0][1]-p[1]) as f32]);
    if u[2].abs() < 1.0 {
        [-1.0, 1.0, 1.0]
    } else {
        [1.0-(u[0]+u[1])/u[2], u[1]/u[2], u[0]/u[2]]
    }
}

fn triangle(image: &mut Image, t: Triangle, color: Pixel) {
    let w = image.get_width() as i32;
    let h = image.get_height() as i32;
    let screen: Point = [w-1, h-1];
    let mut bbn = screen;
    let mut bbx: Point = [0, 0];
    // compute triangle's bounding box
    for i in 0..3 {
        for j in 0..2 {
            bbn[j] = cmp::max(cmp::min(bbn[j], t[i][j]), 0);
            bbx[j] = cmp::min(cmp::max(bbx[j], t[i][j]), screen[j]);
        }
    }
    // iterate over bounding box pixels and check barycentric coordinates are within triangle
    for x in bbn[0]..bbx[0] {
        for y in bbn[1]..bbx[1] {
            let coords = barycentric(t, [x, y]);
            if coords.iter().all(|c| *c >= 0.0) {
                pixel(image, x as u32, y as u32, color)
            }
        }
    }
}

fn draw_poly(image: &mut Image, model: &Model) {
    let w2 = image.get_width()  as f32 / 2.;
    let h2 = image.get_height() as f32 / 2.;
    let light: Vector3<f32> = [0.0, 0.0, -1.0];
    for ref face in &model.faces {
        let mut world: Triangle3f = [[0.0; 3]; 3];
        let mut screen: Triangle = [[0; 2]; 3];
        for i in 0..3 {
            let ref v = model.verts[face[i] as usize];
            world[i] = [v.x, v.y, v.z];
            let x = (v.x+1.)*w2;
            let y = (v.y+1.)*h2;
            screen[i] = [x as i32, y as i32];
        }
        let normal = vec3_normalized(vec3_cross(vec3_sub(world[2], world[0]), vec3_sub(world[1], world[0])));
        let intensity = vec3_dot(normal, light);
        if intensity > 0.0 { // back-face culling
            let luma = (255.0 * intensity) as u8;
            triangle(image, screen, Pixel{r: luma, g: luma, b: luma});
        }
    }
}
