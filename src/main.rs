extern crate image as pimage;

pub mod draw;
pub mod mat;
pub mod model;
pub mod shaders;

fn main() {
    let mut image = pimage::RgbImage::new(1500, 1500);

    let ref model = Box::new(model::load("models/diablo3_pose"));
    draw::draw_poly(&mut image, model);

    // let box ref floor = box model::load("models/floor");
    // draw::draw_poly(&mut image, floor);

    let image_filename = "render.png";
    let _ = image
        .save(image_filename)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", image_filename, e));
}
