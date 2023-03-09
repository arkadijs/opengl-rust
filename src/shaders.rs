extern crate vecmath;
use self::vecmath::{
    row_mat4_transform, vec2_mul, vec3_dot, vec3_scale, Matrix3x2, Matrix4, Vector3,
};

extern crate bmp;
use bmp::Pixel;

extern crate image as pimage;
use self::pimage::RgbImage;

use mat::col_mat3x2_mul_vec3;

pub fn fragment(
    barycentric_coords: Vector3<f32>,
    uv: Matrix3x2<f32>,
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

pub fn vertex(
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