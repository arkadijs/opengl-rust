extern crate vecmath;
use self::vecmath::{
    row_mat4_transform, vec2_mul, vec3_dot, vec3_scale, Matrix3x2, Matrix4, Vector3,
};

extern crate image as pimage;
use self::pimage::{Rgb, RgbImage};

use mat::col_mat3x2_mul_vec3;

pub fn fragment(
    barycentric_coords: Vector3<f32>,
    uv: Matrix3x2<f32>,
    intensity: Vector3<f32>,
    diffuse: &Option<RgbImage>,
    _normal: &Option<RgbImage>,
    _specular: &Option<RgbImage>,
) -> Option<Rgb<u8>> {
    // scale diffuse texture components by interpolated intensity
    let intensity_interpolated = vec3_dot(intensity, barycentric_coords);
    if intensity_interpolated < 0. {
        // light shining on backface
        return None;
    }
    let _scale = |comp: u8| (comp as f32 * intensity_interpolated) as u8;

    let color = match diffuse {
        None => Rgb::<u8>::from([255, 255, 255]),
        Some(texture) => {
            let dw = texture.width() as f32;
            let dh = texture.height() as f32;
            let uv_interpolated = vec2_mul(col_mat3x2_mul_vec3(uv, barycentric_coords), [dw, dh]);
            *texture.get_pixel(uv_interpolated[0] as u32, uv_interpolated[1] as u32)
        }
    };
    let pixel = Rgb([_scale(color[0]), _scale(color[1]), _scale(color[2])]);

    Some(pixel)
}

pub fn vertex(
    vertex: Vector3<f32>,
    normal: Vector3<f32>,
    light: Vector3<f32>,
    transform: Matrix4<f32>,
) -> (Vector3<i32>, f32, f32) {
    let t = row_mat4_transform(transform, [vertex[0], vertex[1], vertex[2], 1.]);
    let perspective_scale = 1. / t[3]; // inverse of perspective divide for convinience
    let screen = vec3_scale([t[0], t[1], t[2]], perspective_scale).map(|n| n as i32);
    let intensity = vec3_dot(normal, light);

    (screen, perspective_scale, intensity)
}
