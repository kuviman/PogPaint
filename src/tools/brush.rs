use super::*;

pub struct Brush {
    ctx: Ctx,
    size: usize,
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
            ctx: ctx.clone(),
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

    fn round_pos(&self, pos: vec2<f32>) -> vec2<f32> {
        if self.size % 2 == 0 {
            pos.map(|x| x.round())
        } else {
            pos.map(|x| (x - 0.5).round() + 0.5)
        }
    }

    fn draw_width(&self) -> f32 {
        let rounded = (self.size as f32 / 2.0).floor() * 2.0;
        (rounded + self.size as f32) / 2.0
    }
}

pub struct BrushStroke {
    prev_draw_pos: vec2<f32>,
    sfx: geng::SoundEffect,
}

impl Drop for BrushStroke {
    fn drop(&mut self) {
        self.sfx.stop();
    }
}

impl Tool for Brush {
    type Stroke = BrushStroke;
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<BrushStroke> {
        if let Some(idx) = state.selected {
            let plane = &mut state.model.planes[idx];
            if let Some(raycast) = plane.raycast(ray) {
                let pos = self.round_pos(raycast.texture_pos);
                plane
                    .texture
                    .draw_line(pos, pos, self.draw_width(), self.actual_color());
                return Some(BrushStroke {
                    prev_draw_pos: pos,
                    sfx: self.ctx.assets.scribble.play(),
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
                plane.texture.draw_line(
                    stroke.prev_draw_pos,
                    pos,
                    self.draw_width(),
                    self.actual_color(),
                );
                stroke.prev_draw_pos = pos;
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

        // Draw preview
        if let Some(ray) = ray {
            if let Some(idx) = state.selected {
                let plane = &state.model.planes[idx];

                let mut preview_plane = Plane {
                    texture: Texture::new(&self.ctx),
                    transform: plane.transform,
                };

                if let Some(raycast) = preview_plane.raycast(ray) {
                    let pos = self.round_pos(raycast.texture_pos);
                    preview_plane.texture.draw_line(
                        pos,
                        pos,
                        self.draw_width(),
                        self.color.map_or(Rgba::WHITE, Into::into),
                    );

                    let offset = {
                        const EPS: f32 = 1e-2;

                        let forward = (state.camera.view_matrix().inverse()
                            * vec4(0.0, 0.0, -1.0, 0.0))
                        .xyz();
                        let plane_up = (plane.transform * vec4(0.0, 0.0, 1.0, 0.0)).xyz();

                        if vec3::dot(forward, plane_up) < 0.0 {
                            EPS
                        } else {
                            -EPS
                        }
                    };
                    let transform =
                        preview_plane.transform * mat4::translate(vec3(0.0, 0.0, offset));
                    if self.color.is_some() {
                        preview_plane
                            .texture
                            .draw(framebuffer, &state.camera, transform)
                    }
                    preview_plane
                        .texture
                        .draw_outline(framebuffer, &state.camera, transform);
                }
            }
        }

        let text = match self.color {
            Some(_) => "brush",
            None => "eraser",
        };
        let text = format!("{text} ({:.1} px)", self.size);
        let font = self.ctx.geng.default_font();
        let text_align = vec2::splat(geng::TextAlign::CENTER);
        let text_measure = font.measure(text.as_str(), text_align).unwrap();
        if let Some(color) = self.color {
            let color: Rgba<f32> = color.into();
            let transform = status_pos * mat3::translate(vec2(text_measure.max.x + 1.5, 0.0));
            ugli::draw(
                framebuffer,
                &self.ctx.shaders.color_2d,
                ugli::DrawMode::TriangleFan,
                &*self.ctx.quad,
                (
                    ugli::uniforms! {
                        u_transform: transform,
                        u_color: color,
                    },
                    ui_camera.uniforms(framebuffer_size),
                ),
                ugli::DrawParameters { ..default() },
            );
        }
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
