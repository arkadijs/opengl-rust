#![feature(box_syntax, box_patterns)]

extern crate image as pimage;

pub mod draw;
pub mod mat;
pub mod model;
pub mod shaders;

fn main() {
    let mut image = pimage::RgbImage::new(500, 500);

    let box ref model = box model::load("models/african_head");
    draw::draw_poly(&mut image, model);

    let box ref floor = box model::load("models/floor");
    draw::draw_poly(&mut image, floor);

    let image_filename = "render.png";
    let _ = image
        .save(image_filename)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", image_filename, e));
}
