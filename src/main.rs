#![feature(box_syntax, box_patterns)]
#![allow(dead_code)]

use std::cmp;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Add;
use std::ops::Mul;
use std::path::Path;
use std::str::FromStr;

extern crate vecmath;
use vecmath::{
    col_mat3x2_row, mat4_id, row_mat4_mul, row_mat4_transform, vec2_mul, vec3_cross, vec3_dot,
    vec3_len, vec3_normalized, vec3_scale, vec3_sub, Matrix3, Matrix3x2, Matrix4, Vector2, Vector3,
};

extern crate bmp;
use bmp::{consts, Image, Pixel};

extern crate image as pimage;
use pimage::RgbImage;

fn main() {
    let box ref model = box load_model("models/african_head");
    let mut image = Image::new(500, 500);
    draw_poly(&mut image, model);
    let bmp = "render.bmp";
    let _ = image
        .save(bmp)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", bmp, e));
}

fn draw_pixel(image: &mut Image, x: u32, y: u32, val: Pixel) {
    let w = image.get_width();
    let h = image.get_height();
    if x < w && y < h {
        image.set_pixel(x, h - y - 1, val)
    }
}

fn draw_line(x0: u32, y0: u32, x1: u32, y1: u32, image: &mut Image, color: Pixel) {
    //println!("{} {} -> {} {}", x0, y0, x1, y1);
    let dx = x1 as i32 - x0 as i32;
    let dy = y1 as i32 - y0 as i32;
    // swap x/y to iterate over longest coord
    let swap = dx.abs() < dy.abs();
    let (_p0, _p1, _q0, _q1, _dp, _dq) = if !swap {
        (x0, x1, y0, y1, dx, dy)
    } else {
        (y0, y1, x0, x1, dy, dx)
    };
    // swap to iterate from smaller `p` coord value to larger
    let (p0, p1, q0, q1) = if _p0 < _p1 {
        (_p0, _p1, _q0, _q1)
    } else {
        (_p1, _p0, _q1, _q0)
    };
    let (dp, dq, q_add) = (_dp.abs(), _dq.abs(), if q0 < q1 { 1 } else { -1 });
    let q_err_add = dq * 2;
    let mut q_err = 0;
    let mut q = q0 as i32;
    for p in p0..(p1 + 1) {
        if !swap {
            draw_pixel(image, p, q as u32, color)
        } else {
            draw_pixel(image, q as u32, p, color)
        }
        q_err += q_err_add;
        if q_err > dp {
            q += q_add;
            q_err -= dp * 2
        }
    }
}

struct Model {
    verts: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    texture: Vec<Vector2<f32>>,
    // a list of triangles, each vertex containing three indices: into `verts`, `texture`, `normals`
    faces: Vec<Vector3<Vector3<usize>>>,
    diffuse: Box<RgbImage>,
}

fn load_model(name: &str) -> Model {
    let obj_file = format!("{}.obj", name);
    let file = BufReader::new(
        File::open(&Path::new(&obj_file))
            .unwrap_or_else(|err| panic!("Cannot open {}: {}", obj_file, err)),
    );
    fn _idx(s: &str) -> usize {
        usize::from_str(s).map(|x| x - 1).unwrap_or(0)
    }
    fn _f32(s: &str) -> f32 {
        f32::from_str(s).unwrap_or(0.)
    }
    fn _mvec(s: &str) -> Vector3<usize> {
        let mut indices = s.splitn(3, '/').take(3).map(|i| _idx(i));
        // [0] is vertex index into "v"/verts
        // [1] is diffuse texture coordinate index into "vt"/texture
        // [2] is normals index into "vn"/normals
        [
            indices.next().unwrap_or(0),
            indices.next().unwrap_or(0),
            indices.next().unwrap_or(0),
        ]
        // s.splitn(1, '/').next().and_then(|_1| usize::from_str::<usize>(_1).ok()).map(|f| f-1).unwrap_or(0)
    }
    let mut verts = vec![];
    let mut normals = vec![];
    let mut faces = vec![];
    let mut texture = vec![];
    for maybe in file.lines() {
        match maybe {
            Err(err) => panic!("I/O error reading {}: {}", obj_file, err),
            Ok(line) => match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                ["v", x, y, z] => verts.push([_f32(x), _f32(y), _f32(z)]),
                ["vn", x, y, z] => normals.push([_f32(x), _f32(y), _f32(z)]),
                ["vt", u, v, _] => texture.push([_f32(u), _f32(v)]),
                ["f", _1, _2, _3] => faces.push([_mvec(_1), _mvec(_2), _mvec(_3)]),
                _ => (),
            },
        }
    }
    let diffuse_file = format!("{}_diffuse.png", name);
    let diffuse = box match pimage::open(&Path::new(&diffuse_file)) {
        Err(err) => panic!("I/O error reading {}: {}", diffuse_file, err),
        Ok(img) => img.into_rgb8(),
    };
    Model {
        verts,
        normals,
        faces,
        texture,
        diffuse,
    }
}

fn draw_wireframe(image: &mut Image, model: &Model) {
    let w2 = image.get_width() as f32 / 2.;
    let h2 = image.get_height() as f32 / 2.;
    for ref face in &model.faces {
        for i in 0..3 {
            let ref v0 = model.verts[face[i][0]];
            let ref v1 = model.verts[face[if i < 2 { i + 1 } else { 0 }][0]];
            let x0 = (v0[0] + 1.) * w2;
            let y0 = (v0[1] + 1.) * h2;
            let x1 = (v1[0] + 1.) * w2;
            let y1 = (v1[1] + 1.) * h2;
            draw_line(
                x0 as u32,
                y0 as u32,
                x1 as u32,
                y1 as u32,
                image,
                consts::WHITE,
            );
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
        [
            (t[2][0] - t[0][0]) as f32,
            (t[1][0] - t[0][0]) as f32,
            (t[0][0] - p[0]) as f32,
        ],
        [
            (t[2][1] - t[0][1]) as f32,
            (t[1][1] - t[0][1]) as f32,
            (t[0][1] - p[1]) as f32,
        ],
    );
    if u[2].abs() < 1. {
        [-1., 1., 1.]
    } else {
        [1. - (u[0] + u[1]) / u[2], u[1] / u[2], u[0] / u[2]]
    }
}

pub fn col_mat3x2_mul_vec3<T>(mat: Matrix3x2<T>, a: Vector3<T>) -> Vector2<T>
where
    T: Copy + Add<T, Output = T> + Mul<T, Output = T>,
{
    [
        vec3_dot(col_mat3x2_row(mat, 0), a),
        vec3_dot(col_mat3x2_row(mat, 1), a),
    ]
}

fn fragment_shader(
    barycentric_coords: Vector3<f32>,
    uv: Trianglet,
    intensity: Vector3<f32>,
    diffuse: &RgbImage,
) -> (Pixel, bool) {
    let dw = diffuse.width() as f32;
    let dh = diffuse.height() as f32;

    let uv_interpolated = vec2_mul(col_mat3x2_mul_vec3(uv, barycentric_coords), [dw, dh]);
    // scale diffuse texture components by interpolated intensity
    let _intensity = vec3_dot(intensity, barycentric_coords);
    let _scale = |comp: u8| (comp as f32 * _intensity) as u8;
    let color = diffuse.get_pixel(
        uv_interpolated[0] as u32,
        (dh - uv_interpolated[1] - 1.) as u32,
    );
    let pixel = Pixel {
        r: _scale(color[0]),
        g: _scale(color[1]),
        b: _scale(color[2]),
    };

    (pixel, false)
}

fn draw_triangle(
    image: &mut Image,
    t: Triangle,
    uv: Trianglet,
    intensity: Vector3<f32>,
    diffuse: &RgbImage,
    zbuffer: &mut Vec<u16>,
) {
    let w = image.get_width() as i32;
    let h = image.get_height() as i32;
    let screen: Point = [w - 1, h - 1];
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
    // iterate over bounding box pixels and check barycentric coordinates are within triangle
    for y in bbn[1]..bbx[1] {
        for x in bbn[0]..bbx[0] {
            let coords = barycentric(t, [x, y]);
            if coords.iter().all(|c| *c >= 0.) {
                let z = (vec3_dot(zcomp, coords) + 0.5) as u16;
                let zi = (y * w + x) as usize;
                if zbuffer[zi] < z {
                    let (pixel, skip) = fragment_shader(coords, uv, intensity, diffuse);
                    if !skip {
                        zbuffer[zi] = z;
                        draw_pixel(image, x as u32, y as u32, pixel);
                    }
                }
            }
        }
    }
}

/*
  Object coordinates
    * Model matrix =>
  World cordinates
    * View matrix (camera) =>
  Eye coordinates
    * Projection matrix (perspective) =>
  Clip coordinates
    * Viewport matrix =>
  Screen coordinates with Z-buffer

  v' = viewport * projection * view * model * v
*/

fn make_viewport(x: u32, y: u32, w: u32, h: u32, depth: u32) -> Matrix4<f32> {
    let mut viewport: Matrix4<f32> = mat4_id();
    let w2 = (w as f32) / 2.;
    let h2 = (h as f32) / 2.;
    let d2 = (depth as f32) / 2.;

    viewport[0][3] = (x as f32) + w2;
    viewport[1][3] = (y as f32) + h2;
    viewport[2][3] = d2;

    viewport[0][0] = w2;
    viewport[1][1] = h2;
    viewport[2][2] = d2;

    viewport
}

fn make_projection(camera: Vector3<f32>, center: Vector3<f32>) -> Matrix4<f32> {
    let mut projection: Matrix4<f32> = mat4_id();
    projection[3][2] = -1. / vec3_len(vec3_sub(camera, center));

    projection
}

fn make_modelview(camera: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let z = vec3_normalized(vec3_sub(camera, center));
    let x = vec3_normalized(vec3_cross(up, z));
    let y = vec3_normalized(vec3_cross(z, x));
    let mut inverse = mat4_id();
    let mut translation = mat4_id();
    for i in 0..3 {
        inverse[0][i] = x[i];
        inverse[1][i] = y[i];
        inverse[2][i] = z[i];
        translation[i][3] = -center[i]
    }
    let modelview = row_mat4_mul(inverse, translation);

    modelview
}

fn vertex_shader(
    vertex: Vector3<f32>,
    normal: Vector3<f32>,
    light: Vector3<f32>,
    transform: Matrix4<f32>,
) -> (Vector3<i32>, f32) {
    let t = row_mat4_transform(transform, [vertex[0], vertex[1], vertex[2], 1.]);
    let screen = vec3_scale([t[0], t[1], t[2]], 1. / t[3]).map(|n| n as i32);
    let intensity = vec3_dot(normal, light);

    (screen, intensity)
}

fn draw_poly(image: &mut Image, model: &Model) {
    let w = image.get_width();
    let h = image.get_height();

    let center: Vector3<f32> = [0., 0., 0.];
    let camera: Vector3<f32> = [1., 1., 3.];
    let light: Vector3<f32> = vec3_normalized([1., -1., 1.]);

    let viewport: Matrix4<f32> = make_viewport(w / 8, h / 8, w * 3 / 4, h * 3 / 4, u16::MAX as u32);
    let projection: Matrix4<f32> = make_projection(camera, center);
    let modelview: Matrix4<f32> = make_modelview(camera, center, [0., 1., 0.]);
    let transform = row_mat4_mul(row_mat4_mul(viewport, projection), modelview);

    let zsize = (w * h) as usize;
    let mut zbuffer = Vec::<u16>::with_capacity(zsize);
    zbuffer.resize(zsize, 0);

    for ref face in &model.faces {
        let mut screen: Triangle = [[0; 3]; 3];
        let mut texture: Trianglet = [[0.; 2]; 3];
        let mut intensity: Vector3<f32> = [0.; 3];
        for i in 0..3 {
            let ref indices = face[i];
            (screen[i], intensity[i]) = vertex_shader(
                model.verts[indices[0]],   // [0] index into 3d vertex coords
                model.normals[indices[2]], // [2] index into 3d vertex normal
                light,
                transform,
            );
            texture[i] = model.texture[indices[1]]; // [1] index into 2d texture coords
        }
        draw_triangle(
            image,
            screen,
            texture,
            intensity,
            &model.diffuse,
            &mut zbuffer,
        );
    }
}
