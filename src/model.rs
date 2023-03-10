use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

extern crate vecmath;
use self::vecmath::{Vector2, Vector3};

extern crate image as pimage;
use self::pimage::error::ImageError;
use self::pimage::RgbImage;

pub struct FaceVertexIndices {
    pub vert: usize,
    pub tex: usize,
    pub norm: usize,
}

pub struct Model {
    pub verts: Vec<Vector3<f32>>,
    pub texture: Vec<Vector2<f32>>,
    pub vert_normals: Vec<Vector3<f32>>,
    // a list of triangles, each vertex containing three indices: into `verts`, `texture`, `normals`
    pub faces: Vec<Vector3<FaceVertexIndices>>,
    pub diffuse: Box<Option<RgbImage>>,
    pub normal: Box<Option<RgbImage>>,
    pub specular: Box<Option<RgbImage>>,
    pub glow: Box<Option<RgbImage>>,
}

pub fn load(name: &str) -> Model {
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
    fn _mvec(s: &str) -> FaceVertexIndices {
        let mut indices = s.splitn(3, '/').take(3).map(|i| _idx(i));
        // [0] is vertex index into "v"/verts
        // [1] is diffuse texture coordinate index into "vt"/texture
        // [2] is normals index into "vn"/normals
        FaceVertexIndices {
            vert: indices.next().unwrap_or(0),
            tex: indices.next().unwrap_or(0),
            norm: indices.next().unwrap_or(0),
        }
    }
    let mut verts = vec![];
    let mut texture = vec![];
    let mut vert_normals = vec![];
    let mut faces = vec![];
    for maybe in file.lines() {
        match maybe {
            Err(err) => panic!("I/O error reading {}: {}", obj_file, err),
            Ok(line) => match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                ["v", x, y, z] => verts.push([_f32(x), _f32(y), _f32(z)]),
                ["vn", x, y, z] => vert_normals.push([_f32(x), _f32(y), _f32(z)]),
                ["vt", u, v] | ["vt", u, v, _] => texture.push([_f32(u), _f32(v)]),
                ["f", _1, _2, _3] => faces.push([_mvec(_1), _mvec(_2), _mvec(_3)]),
                _ => (),
            },
        }
    }

    let [diffuse, normal, specular, glow] = ["diffuse", "nm_tangent", "spec", "glow"].map(|kind| {
        let filename = format!("{}_{}.png", name, kind);
        box match pimage::open(&Path::new(&filename)) {
            Ok(img) => Some(img.into_rgb8()),
            Err(err) => match err {
                ImageError::IoError(ioerr) => match ioerr.kind() {
                    std::io::ErrorKind::NotFound => None,
                    _ => panic!("I/O error reading {}: {}", filename, ioerr),
                },
                _ => panic!("Image error reading {}: {}", filename, err),
            },
        }
    });

    Model {
        verts,
        texture,
        vert_normals,
        faces,
        diffuse,
        normal,
        specular,
        glow,
    }
}
