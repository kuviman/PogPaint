use super::*;

pub trait ItemWheel {
    fn item_count(&self) -> usize;
    fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn geng::AbstractCamera2d,
        transform: mat3<f32>,
        items: &[Item],
    );
    fn select(&self, item: usize, app: &mut App);
}

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
    Items(Box<dyn ItemWheel>),
    Continuous(Box<dyn ContiniousWheel>),
}

pub struct Wheel {
    pub pos: vec2<f32>,
    pub typ: WheelType,
}

pub struct Item {
    pub angle: Angle<f32>,
    pub local_transform: mat3<f32>,
    pub hovered: bool,
}

pub fn items(app: &App, item_count: usize, hover: Option<Angle<f32>>) -> Vec<Item> {
    let mut items: Vec<Item> = (0..item_count)
        .map(move |item| {
            let angle = Angle::from_degrees(360.0 * item as f32 / item_count as f32);
            let r = (app.ctx.config.wheel.inner_radius + 1.0) / 2.0;
            let size = (2.0 * f32::PI * r) / item_count as f32 / 2.0;
            let local_transform =
                mat3::translate(vec2(r, 0.0).rotate(angle)) * mat3::scale_uniform(size);
            Item {
                angle,
                local_transform,
                hovered: false,
            }
        })
        .collect();
    if let Some(hover) = hover {
        items
            .iter_mut()
            .min_by_key(|item| (item.angle - hover).normalized_pi().abs().map(r32))
            .unwrap()
            .hovered = true;
    }
    items
}

pub fn calculate_hover(app: &App, wheel: &Wheel) -> Option<Angle<f32>> {
    app.ctx.geng.window().cursor_position().and_then(|pos| {
        let pos = app
            .ui_camera
            .screen_to_world(app.framebuffer_size, pos.map(|x| x as f32));
        let delta = pos - wheel.pos;
        if delta.len() / app.ctx.config.wheel.size < app.ctx.config.wheel.inner_radius {
            return None;
        }
        Some(delta.arg())
    })
}

pub fn draw(app: &App, wheel: &Wheel, framebuffer: &mut ugli::Framebuffer) {
    let hover = calculate_hover(app, wheel);
    let transform = mat3::translate(wheel.pos) * mat3::scale_uniform(app.ctx.config.wheel.size);
    match &wheel.typ {
        WheelType::Items(typ) => {
            typ.draw(
                framebuffer,
                &app.ui_camera,
                transform,
                &items(app, typ.item_count(), hover),
            );
        }
        WheelType::Continuous(typ) => {
            typ.draw(framebuffer, &app.ui_camera, transform, hover);
        }
    }
}
