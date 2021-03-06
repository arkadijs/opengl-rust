#![feature(box_syntax, box_patterns, convert, float_from_str_radix, slice_patterns, vec_resize)]
#![allow(dead_code)]

use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;

extern crate vecmath;
use vecmath::{Vector2, Vector3, Matrix3, Matrix3x2, vec3_sub, vec3_cross, vec3_dot, vec3_normalized};

extern crate bmp;
use bmp::{Image, Pixel, consts};

extern crate image as pimage;
use pimage::RgbImage;

fn main() {
    let box ref model = box load("models/african_head");
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

#[derive(Debug)]
struct UV {
    u: f32, v: f32
}

struct Model {
    verts: Vec<Vec3f>,
    texture: Vec<UV>,
    // a list of triangles, each vertex containing two indices: into `verts` and `texture`
    faces: Vec<Vector3<Vector2<usize>>>,
    diffuse: Box<RgbImage>
}

fn load(name: &str) -> Model {
    let obj_file = format!("{}.obj", name);
    let file = BufReader::new(File::open(&Path::new(&obj_file)).unwrap_or_else(|err| { panic!("Cannot open {}: {}", obj_file, err) }));
    fn _idx(s: &str) -> usize { usize::from_str_radix(s, 10).map(|x| x-1).unwrap_or(0) };
    fn _f32(s: &str) -> f32   {   f32::from_str_radix(s, 10).unwrap_or(0.) };
    fn _mvec(s: &str) -> Vector2<usize> {
        let mut indices = s.splitn(3, '/').take(2).map(|i| _idx(i));
        // [0] is vertex index into "v"/verts, [1] is diffuse texture coordinate index into "vt"/texture
        [indices.next().unwrap_or(0), indices.next().unwrap_or(0)]
        // s.splitn(1, '/').next().and_then(|_1| num::from_str_radix::<usize>(_1, 10).ok()).map(|f| f-1).unwrap_or(0)
    };
    let mut verts = vec![];
    let mut faces = vec![];
    let mut texture = vec![];
    for maybe in file.lines() {
        match maybe {
            Err(err) => panic!("I/O error reading {}: {}", obj_file, err),
            Ok(line) => {
                match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                    ["v", x, y, z]    => verts.push(Vec3f{ x: _f32(x), y: _f32(y), z: _f32(z) }),
                    ["vt", u, v, _]   => texture.push(UV{ u: _f32(u), v: _f32(v) }),
                    ["f", _1, _2, _3] => faces.push([_mvec(_1), _mvec(_2), _mvec(_3)]),
                    _ => () ,
                }
            }
        }
    }
    let diffuse_file = format!("{}_diffuse.png", name);
    let diffuse = box match pimage::open(&Path::new(&diffuse_file)) {
        Err(err) => panic!("I/O error reading {}: {}", diffuse_file, err),
        Ok(img) => img.to_rgb()
    };
    Model{ verts: verts, faces: faces, texture: texture, diffuse: diffuse }
}

fn draw_wireframe(image: &mut Image, model: &Model) {
    let w2 = image.get_width()  as f32 / 2.;
    let h2 = image.get_height() as f32 / 2.;
    for ref face in &model.faces {
        for i in 0..3 {
            let ref v0 = model.verts[face[i][0]];
            let ref v1 = model.verts[face[if i < 2 {i+1} else {0}][0]];
            let x0 = (v0.x+1.)*w2;
            let y0 = (v0.y+1.)*h2;
            let x1 = (v1.x+1.)*w2;
            let y1 = (v1.y+1.)*h2;
            line(x0 as u32, y0 as u32, x1 as u32, y1 as u32, image, consts::WHITE);
        }
    }
}

type Point = Vector2<i32>; // pixel on the screen
type Triangle = Matrix3<i32>; // triangle in screen pixels
type Trianglet = Matrix3x2<f32>; // triangle texture coords 0-1.0
type Trianglef = Matrix3<f32>; // triangle in world coordinates
type Baricentric = Vector3<f32>; // pixel baricentric coordinates in triangle screen coords

fn barycentric(t: Triangle, p: Point) -> Baricentric {
    let u = vec3_cross(
        [(t[2][0]-t[0][0]) as f32, (t[1][0]-t[0][0]) as f32, (t[0][0]-p[0]) as f32],
        [(t[2][1]-t[0][1]) as f32, (t[1][1]-t[0][1]) as f32, (t[0][1]-p[1]) as f32]);
    if u[2].abs() < 1. {
        [-1., 1., 1.]
    } else {
        [1.-(u[0]+u[1])/u[2], u[1]/u[2], u[0]/u[2]]
    }
}

fn triangle(image: &mut Image, t: Triangle, uv: Trianglet, intensity: f32, diffuse: &RgbImage, zbuffer: &mut Vec<u16>) {
    let dw = diffuse.width() as f32;
    let dh = diffuse.height() as f32;
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
    // z-components of triangle vertices for pixel z-coordinate interpolation
    let zcomp = [t[0][2] as f32, t[1][2] as f32, t[2][2] as f32];
    // u,v texture coordinates for interpolation
    let ucomp = [uv[0][0], uv[1][0], uv[2][0]];
    let vcomp = [uv[0][1], uv[1][1], uv[2][1]];
    // scale diffuse texture components by intensity
    let _scale = |comp: u8| { (comp as f32 * intensity) as u8 };
    // iterate over bounding box pixels and check barycentric coordinates are within triangle
    for y in bbn[1]..bbx[1] {
        for x in bbn[0]..bbx[0] {
            let coords = barycentric(t, [x, y]);
            if coords.iter().all(|c| *c >= 0.) {
                let z = (vec3_dot(zcomp, coords) + 0.5) as u16;
                let zi = (y*w+x) as usize;
                if zbuffer[zi] < z {
                    zbuffer[zi] = z;
                    let u = dw * vec3_dot(ucomp, coords);
                    let v = dh * vec3_dot(vcomp, coords);
                    let c = diffuse.get_pixel(u as u32, (dh - v - 1.) as u32);
                    pixel(image, x as u32, y as u32, Pixel{ r: _scale(c[0]), g: _scale(c[1]), b: _scale(c[2]) });
                }
            }
        }
    }
}

fn draw_poly(image: &mut Image, model: &Model) {
    let w = image.get_width();
    let h = image.get_height();
    let w2 = w as f32 / 2.;
    let h2 = h as f32 / 2.;
    let light: Vector3<f32> = [0., 0., -1.];
    let zsize = (w*h) as usize;
    let mut zbuffer = Vec::with_capacity(zsize);
    zbuffer.resize(zsize, 0);
    for ref face in &model.faces {
        let mut world: Trianglef = [[0.; 3]; 3];
        let mut screen: Triangle = [[0; 3]; 3];
        let mut texture: Trianglet = [[0.; 2]; 3];
        for i in 0..3 {
            let ref indices = face[i];
            let ref v = model.verts[indices[0]]; // [0] index into 3d vertex coords
            world[i] = [v.x, v.y, v.z];
            let x = (v.x+1.)*w2;
            let y = (v.y+1.)*h2;
            let z = (v.z+1.)*((std::u16::MAX/2) as f32);
            screen[i] = [x as i32, y as i32, z as i32];
            let ref t = model.texture[indices[1]]; // [1] index into 2d texture coords
            texture[i] = [t.u, t.v];
        }
        let normal = vec3_normalized(vec3_cross(vec3_sub(world[2], world[0]), vec3_sub(world[1], world[0])));
        let intensity = vec3_dot(normal, light);
        if intensity > 0. { // back-face culling
            triangle(image, screen, texture, intensity, &model.diffuse, &mut zbuffer);
        }
    }
}
