use super::*;

pub struct Heightmap {
    ctx: Ctx,
    size: usize,
}

impl Heightmap {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            size: ctx.config.default_brush.size,
        }
    }

    fn round_pos(&self, pos: vec2<f32>) -> vec2<f32> {
        pos.map(|x| x.round())
    }

    fn draw_width(&self) -> f32 {
        let rounded = (self.size as f32 / 2.0).floor() * 2.0;
        (rounded + self.size as f32) / 2.0
    }

    fn paint(&self, heightmap: &mut Option<crate::Heightmap>, pos: vec2<f32>, delta_time: f32) {
        let heightmap = heightmap.get_or_insert_with(crate::Heightmap::new);
        let bb = Aabb2::point(pos.map(|x| x.round() as i32)).extend_uniform(self.size as i32);
        heightmap.ensure_bounds(bb);
        for p in bb.points() {
            let x = (1.0 - (p.map(|x| x as f32) - pos).len() / self.size as f32).max(0.0);
            *heightmap.get_mut(p).unwrap() +=
                self.ctx.config.heightmap.change_speed * delta_time * x;
        }
    }
}

pub struct BrushStroke {
    prev_draw_pos: vec2<f32>,
    timer: Timer,
}

impl Tool for Heightmap {
    type Stroke = BrushStroke;
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<BrushStroke> {
        if let Some(idx) = state.selected {
            let plane = &mut state.model.planes[idx];
            if let Some(raycast) = plane.raycast(ray) {
                let pos = self.round_pos(raycast.texture_pos);
                return Some(BrushStroke {
                    prev_draw_pos: pos,
                    timer: Timer::new(),
                });
            }
        }
        None
    }
    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {
        if let Some(idx) = state.selected {
            let plane = &mut state.model.planes[idx];
            if let Some(raycast) = plane.raycast(ray) {
                let pos = self.round_pos(raycast.texture_pos);
                self.paint(
                    &mut plane.heightmap,
                    pos,
                    stroke.timer.tick().as_secs_f64() as f32,
                );
            }
        }
    }
    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray) {}

    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        _stroke: Option<&mut Self::Stroke>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);

        // TODO Draw preview

        let text = "heightmap";
        let text = format!("{text} ({:.1} px)", self.size);
        let font = self.ctx.geng.default_font();
        let text_align = vec2::splat(geng::TextAlign::CENTER);
        let text_measure = font.measure(text.as_str(), text_align).unwrap();
        font.draw(
            framebuffer,
            ui_camera,
            text.as_str(),
            text_align,
            status_pos,
            Rgba::WHITE,
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyPress { key } = event {
            let keys = &self.ctx.keys.brush;
            if keys.decrease_size.matches(&event, &self.ctx) {
                self.size = (self.size - 1).max(1);
            }
            if keys.increase_size.matches(&event, &self.ctx) {
                self.size += 1;
            }
            for (size, size_key) in (1..).zip(&keys.sizes) {
                if size_key.matches(&event, &self.ctx) {
                    self.size = size;
                }
            }
        }
    }
}
