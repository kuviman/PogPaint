use super::*;

pub struct ColorPicker {
    ctx: Ctx,
}

impl ColorPicker {
    pub fn new(ctx: &Ctx) -> Self {
        Self { ctx: ctx.clone() }
    }
    fn find(&self, state: &State, ray: Ray) -> Option<Rgba<f32>> {
        let mut closest = None;
        for plane in &state.model.planes {
            if let Some(raycast) = plane.raycast(ray) {
                let color = plane.texture.color_at(raycast.texture_pos);
                if color.a == 0.0 {
                    continue;
                }
                let new = (raycast.t, color);
                closest = match closest {
                    Some(current) => {
                        Some(std::cmp::min_by_key(current, new, |&(t, _color)| r32(t)))
                    }
                    None => Some(new),
                };
            }
        }
        closest.map(|(_t, color)| color)
    }
}

impl Tool for ColorPicker {
    type Stroke = ();
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<()> {
        if let Some(color) = self.find(state, ray) {
            state.color = color;
        }
        None
    }
    fn resume(&mut self, _stroke: &mut Self::Stroke, _state: &mut State, _ray: Ray) {}
    fn end(&mut self, _stroke: Self::Stroke, _state: &mut State, _ray: Ray) {}

    fn draw(
        &mut self,
        _framebuffer: &mut ugli::Framebuffer,
        _ray: Option<Ray>,
        _stroke: Option<&mut Self::Stroke>,
        _state: &mut State,
        _ui_camera: &dyn AbstractCamera2d,
        _status_pos: mat3<f32>,
    ) {
    }
}
