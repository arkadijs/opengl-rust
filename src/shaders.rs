extern crate vecmath;
use self::vecmath::{
    row_mat4_transform, vec2_mul, vec3_dot, vec3_normalized, vec3_scale, Matrix3x2, Matrix4,
    Vector3,
};

extern crate image as pimage;
use self::pimage::{Pixel, Rgb, RgbImage};

use mat::col_mat3x2_mul_vec3;

pub fn fragment(
    barycentric_coords: Vector3<f32>,
    uv: Matrix3x2<f32>,
    intensity: Vector3<f32>,
    modelviewprojection: Matrix4<f32>,
    modelviewprojection_transposed_inverted: Matrix4<f32>,
    light: Vector3<f32>,
    diffuse: &Option<RgbImage>,
    normal: &Option<RgbImage>,
    _specular: &Option<RgbImage>,
) -> Option<Rgb<u8>> {
    // scale diffuse texture components by interpolated intensity
    let mut intensity_interpolated = vec3_dot(intensity, barycentric_coords);
    // if intensity_interpolated < 0. {
    //     // light shining on backface
    //     return None;
    // }

    let interpolated_uv_coords = col_mat3x2_mul_vec3(uv, barycentric_coords);
    let _interpolate = |texture: &RgbImage| {
        let dw = texture.width() as f32;
        let dh = texture.height() as f32;
        let [u, v] = vec2_mul(interpolated_uv_coords, [dw, dh]).map(|n| n as u32);
        *texture.get_pixel(u, v)
    };

    let color = match diffuse {
        None => Rgb::<u8>::from([255, 255, 255]),
        Some(diffuse_texture) => _interpolate(diffuse_texture),
    };

    if let Some(normal_texture) = normal {
        let norm = _interpolate(normal_texture);

        let n = row_mat4_transform(
            modelviewprojection_transposed_inverted,
            [norm[0] as f32, norm[1] as f32, norm[2] as f32, 0.],
        );
        let norm_transformed = vec3_normalized([n[0], n[1], n[2]]);

        let l = row_mat4_transform(modelviewprojection, [light[0], light[1], light[2], 0.]);
        let light_transformed = vec3_normalized([l[0], l[1], l[2]]);

        intensity_interpolated = f32::max(vec3_dot(norm_transformed, light_transformed), 0.);
    }

    let pixel = color.map(|comp: u8| (comp as f32 * intensity_interpolated) as u8);

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
    let intensity = vec3_dot(normal, vec3_normalized(light));

    (screen, perspective_scale, intensity)
}
