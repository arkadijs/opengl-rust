#![feature(box_syntax, box_patterns)]

extern crate bmp;

pub mod draw;
pub mod mat;
pub mod model;
pub mod shaders;

fn main() {
    let box ref model = box model::load("models/african_head");
    let mut image = bmp::Image::new(500, 500);
    draw::draw_poly(&mut image, model);
    let image_filename = "render.bmp";
    let _ = image
        .save(image_filename)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", image_filename, e));
}
