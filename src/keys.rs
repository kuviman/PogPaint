use super::*;

#[derive(Deserialize)]
pub struct Camera {
    pub forward: geng::Key,
    pub left: geng::Key,
    pub right: geng::Key,
    pub back: geng::Key,
    pub up: geng::Key,
    pub down: geng::Key,
    pub look: geng::Key,
}

#[derive(Deserialize)]
pub struct ToolKeys {
    pub brush: Option<KeyBind>,
    pub eraser: Option<KeyBind>,
    pub transform: Option<KeyBind>,
    pub pick: Option<KeyBind>,
    pub create: Option<KeyBind>,
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
    pub increase_size: KeyBind,
    pub decrease_size: KeyBind,
    pub sizes: Vec<KeyBind>,
}

#[derive(Deserialize)]
pub struct Config {
    pub switch_plane: KeyBind,
    pub palette: KeyBind,
    pub first_person: KeyBind,
    pub camera: Camera,
    pub tools: Tools,
    pub precision: Precision,
    pub brush: Brush,
}
