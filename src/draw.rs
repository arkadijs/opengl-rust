use std::cmp;

extern crate vecmath;
use self::vecmath::{
    mat4_inv, mat4_transposed, row_mat4_mul, vec3_cross, vec3_dot, vec3_mul, vec3_scale, Matrix3,
    Matrix3x2, Matrix4, Vector2, Vector3,
};

extern crate image as pimage;
use self::pimage::{Rgb, RgbImage};

use mat::{make_modelview, make_projection, make_viewport};
use model::Model;
use shaders;

pub fn draw_pixel(image: &mut RgbImage, x: u32, y: u32, val: Rgb<u8>) {
    let (w, h) = image.dimensions();
    if x < w && y < h {
        image.put_pixel(x, h - y - 1, val)
    }
}

pub fn draw_line(x0: u32, y0: u32, x1: u32, y1: u32, image: &mut RgbImage, color: Rgb<u8>) {
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

fn _draw_wireframe(image: &mut RgbImage, model: &Model) {
    let w2 = image.width() as f32 / 2.;
    let h2 = image.height() as f32 / 2.;
    for ref face in &model.faces {
        for i in 0..3 {
            let ref v0 = model.verts[face[i].vert];
            let ref v1 = model.verts[face[if i < 2 { i + 1 } else { 0 }].vert];
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
                Rgb::<u8>::from([255, 255, 255]),
            );
        }
    }
}

type Point = Vector2<i32>; // pixel on the screen
type Triangle = Matrix3<i32>; // triangle in screen pixels
type Trianglet = Matrix3x2<f32>; // triangle texture coords 0-1.0
type Baricentric = Vector3<f32>; // pixel baricentric coordinates in triangle screen coords

fn barycentric(t: Triangle, p: Point) -> Option<Baricentric> {
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
        None
    } else {
        let maybe = [1. - (u[0] + u[1]) / u[2], u[1] / u[2], u[0] / u[2]];
        if maybe.iter().all(|&c| c >= 0.) {
            Some(maybe)
        } else {
            None
        }
    }
}

fn draw_triangle(
    image: &mut RgbImage,
    t: Triangle,
    uv: Trianglet,
    perspective_scale: Vector3<f32>,
    intensity: Vector3<f32>,
    modelviewprojection: Matrix4<f32>,
    modelviewprojection_transposed_inverted: Matrix4<f32>,
    light: Vector3<f32>,
    diffuse: &Option<RgbImage>,
    normal: &Option<RgbImage>,
    specular: &Option<RgbImage>,
    zbuffer: &mut Vec<u16>,
) {
    let w = image.width() as i32;
    let h = image.height() as i32;
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
            if let Some(coords) = barycentric(t, [x, y]) {
                let z = (vec3_dot(zcomp, coords) + 0.5) as u16;
                let zi = (y * w + x) as usize;
                if zbuffer[zi] < z {
                    // for perspective divide aware interpolation
                    let c = vec3_mul(coords, perspective_scale);
                    let clip_coords = vec3_scale(c, 1. / c.iter().sum::<f32>());

                    if let Some(pixel) = shaders::fragment(
                        clip_coords,
                        uv,
                        intensity,
                        modelviewprojection,
                        modelviewprojection_transposed_inverted,
                        light,
                        diffuse,
                        normal,
                        specular,
                    ) {
                        zbuffer[zi] = z;
                        draw_pixel(image, x as u32, y as u32, pixel);
                    }
                }
            }
        }
    }
}

pub fn draw_poly(image: &mut RgbImage, model: &Model) {
    let w = image.width();
    let h = image.height();

    let center: Vector3<f32> = [0., 0., 0.];
    let camera: Vector3<f32> = [1., 1., 3.];
    let light: Vector3<f32> = [2., 2., 1.];

    let viewport: Matrix4<f32> = make_viewport(w / 8, h / 8, w * 3 / 4, h * 3 / 4, u16::MAX as u32);
    let projection: Matrix4<f32> = make_projection(camera, center);
    let modelview: Matrix4<f32> = make_modelview(camera, center, [0., 1., 0.]);
    let modelviewprojection = row_mat4_mul(projection, modelview);
    let modelviewprojection_transposed_inverted = mat4_inv(mat4_transposed(modelviewprojection));
    let transform = row_mat4_mul(viewport, modelviewprojection);

    let zsize = (w * h) as usize;
    let mut zbuffer = Vec::<u16>::with_capacity(zsize);
    zbuffer.resize(zsize, 0);

    for ref face in &model.faces {
        let mut screen: Triangle = [[0; 3]; 3];
        let mut texture: Trianglet = [[0.; 2]; 3];
        let mut perspective_scale: Vector3<f32> = [0.; 3];
        let mut intensity: Vector3<f32> = [0.; 3];
        for i in 0..3 {
            let ref face_vertex = face[i];
            (screen[i], perspective_scale[i], intensity[i]) = shaders::vertex(
                model.verts[face_vertex.vert],
                model.vert_normals[face_vertex.norm],
                light,
                transform,
            );
            texture[i] = model.texture[face_vertex.tex];
        }
        // no-light backface culling
        // if intensity[0] < 0. {
        //     continue;
        // }
        draw_triangle(
            image,
            screen,
            texture,
            perspective_scale,
            intensity,
            modelviewprojection,
            modelviewprojection_transposed_inverted,
            light,
            &model.diffuse,
            &model.normal,
            &model.specular,
            &mut zbuffer,
        );
    }
}
