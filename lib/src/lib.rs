use geng::prelude::*;

pub mod file_format;
mod plane;
mod texture;

pub use plane::*;
pub use texture::*;

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
