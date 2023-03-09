use std::ops::{Add, Mul};

extern crate vecmath;
use self::vecmath::{
    col_mat3x2_row, mat4_id, row_mat4_mul, vec3_cross, vec3_dot, vec3_len, vec3_normalized,
    vec3_sub, Matrix3x2, Matrix4, Vector2, Vector3,
};

pub fn col_mat3x2_mul_vec3<T>(mat: Matrix3x2<T>, a: Vector3<T>) -> Vector2<T>
where
    T: Copy + Add<T, Output = T> + Mul<T, Output = T>,
{
    [
        vec3_dot(col_mat3x2_row(mat, 0), a),
        vec3_dot(col_mat3x2_row(mat, 1), a),
    ]
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

pub fn make_viewport(x: u32, y: u32, w: u32, h: u32, depth: u32) -> Matrix4<f32> {
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

pub fn make_projection(camera: Vector3<f32>, center: Vector3<f32>) -> Matrix4<f32> {
    let mut projection: Matrix4<f32> = mat4_id();
    projection[3][2] = -1. / vec3_len(vec3_sub(camera, center));

    projection
}

pub fn make_modelview(
    camera: Vector3<f32>,
    center: Vector3<f32>,
    up: Vector3<f32>,
) -> Matrix4<f32> {
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
