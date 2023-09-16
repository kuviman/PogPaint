use super::*;

#[derive(Deserialize)]
pub struct Camera {
    pub forward: geng::Key,
    pub left: geng::Key,
    pub right: geng::Key,
    pub back: geng::Key,
    pub up: geng::Key,
    pub down: geng::Key,
}

#[derive(Deserialize)]
pub struct ToolKeys {
    pub brush: Option<geng::Key>,
    pub eraser: Option<geng::Key>,
    pub transform: Option<geng::Key>,
    pub pick: Option<geng::Key>,
    pub create: Option<geng::Key>,
}

#[derive(Deserialize)]
pub struct Tools {
    #[serde(flatten)]
    pub switch: ToolKeys,
    pub temp: ToolKeys,
}

#[derive(Deserialize)]
pub struct Precision {
    pub pixel: geng::Key,
    pub unbounded: geng::Key,
}

#[derive(Deserialize)]
pub struct Brush {
    pub increase_size: geng::Key,
    pub decrease_size: geng::Key,
    pub sizes: Vec<geng::Key>,
}

#[derive(Deserialize)]
pub struct Config {
    pub switch_plane: geng::Key,
    pub palette: geng::Key,
    pub first_person: geng::Key,
    pub camera: Camera,
    pub tools: Tools,
    pub precision: Precision,
    pub brush: Brush,
}
