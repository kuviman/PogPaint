use super::*;

#[derive(Deserialize)]
pub struct Ui {
    pub fov: f32,
}

#[derive(Deserialize)]
pub struct Wheel {
    pub size: f32,
    pub inner_radius: f32,
}

#[derive(Deserialize)]
pub struct Camera {
    pub fov: f32,
    pub rotation: f32,
    pub attack: f32,
    pub distance: f32,
    pub sensitivity: f32,
    pub move_speed: f32,
    pub zoom_speed: f32,
}

#[derive(Deserialize)]
pub struct Gizmo {
    pub width: f32,
    pub outline: f32,
    pub size: f32,
}

#[derive(Deserialize)]
pub struct Grid {
    pub cell_size: f32,
    pub line_count: isize,
    pub color: Rgba<f32>,
}

#[derive(Deserialize)]
pub struct DefaultBrush {
    pub size: usize,
    pub color: Rgba<f32>,
}

#[derive(Deserialize)]
pub enum StatusPos {
    Top,
    Bottom,
}

#[derive(Deserialize)]
pub struct Status {
    pub pos: StatusPos,
    pub width: f32,
}

#[derive(Deserialize)]
pub struct Heightmap {
    pub min: f32,
    pub max: f32,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub max_texture_size: usize,
    pub gizmo: Gizmo,
    pub camera: Camera,
    pub default_brush: DefaultBrush,
    pub background_color: Rgba<f32>,
    pub wheel: Wheel,
    pub ui: Ui,
    pub grid: Grid,
    pub default_palette: Vec<Rgba<f32>>,
    pub status: Status,
    pub heightmap: Heightmap,
}
