use array2d::Array2D;
use geng::prelude::*;

pub mod file_format;
mod heightmap;
mod plane;
mod texture;

pub use heightmap::*;
pub use plane::*;
pub use texture::*;

#[derive(Clone)]
pub struct Model {
    ugli: Ugli,
    pub planes: Vec<Plane>,
}

impl Model {
    pub fn new(ugli: &Ugli) -> Self {
        Self {
            ugli: ugli.clone(),
            planes: vec![],
        }
    }
}
