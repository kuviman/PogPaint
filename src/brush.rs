use super::*;

pub struct Brush {
    size: f32,
    /// None means eraser
    color: Option<Hsla<f32>>,
}

impl Brush {
    pub fn default(ctx: &Ctx) -> Self {
        Self::new(ctx, ctx.config.default_brush.color)
    }

    pub fn eraser(ctx: &Ctx) -> Self {
        Self::new_impl(ctx, None)
    }

    pub fn new(ctx: &Ctx, color: Rgba<f32>) -> Self {
        Self::new_impl(ctx, Some(color))
    }

    fn new_impl(ctx: &Ctx, color: Option<Rgba<f32>>) -> Self {
        Self {
            size: ctx.config.default_brush.size,
            color: color.map(Into::into),
        }
    }

    fn actual_color(&self) -> Rgba<f32> {
        match self.color {
            Some(color) => color.into(),
            None => Rgba::TRANSPARENT_BLACK,
        }
    }
}

pub struct BrushStroke {
    prev_draw_pos: vec2<f32>,
}

impl Tool for Brush {
    type Stroke = BrushStroke;
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<BrushStroke> {
        if let Some(idx) = state.selected {
            let plane = &mut state.planes[idx];
            if let Some(pos) = plane.raycast(ray) {
                plane
                    .texture
                    .draw_line(pos.texture, pos.texture, self.size, self.actual_color());
                state.start_scribble();
                return Some(BrushStroke {
                    prev_draw_pos: pos.texture,
                });
            }
        }
        None
    }
    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {
        if let Some(idx) = state.selected {
            let plane = &mut state.planes[idx];
            if let Some(pos) = plane.raycast(ray) {
                plane.texture.draw_line(
                    stroke.prev_draw_pos,
                    pos.texture,
                    self.size,
                    self.actual_color(),
                );
                stroke.prev_draw_pos = pos.texture;
            }
        }
    }
    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray) {
        if let Some(mut sfx) = state.scribble.take() {
            sfx.stop();
        }
    }

    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        stroke: Option<&mut Self::Stroke>,
        state: &mut State,
    ) {
        // TODO
    }
}
