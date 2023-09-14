use super::*;

pub trait ContiniousWheel {
    fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn geng::AbstractCamera2d,
        transform: mat3<f32>,
        hover: Option<Angle<f32>>,
    );
    fn select(&self, hover: Angle<f32>, app: &mut App);
}

pub enum WheelType {
    Items,
    Continious(Box<dyn ContiniousWheel>),
}

pub struct Wheel {
    pub pos: vec2<f32>,
    pub typ: WheelType,
}
