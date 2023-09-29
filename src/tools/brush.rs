use super::*;

pub struct Brush {
    ctx: Ctx,
    size: usize,
    eraser: bool,
}

impl Brush {
    pub fn default(ctx: &Ctx) -> Self {
        Self::new(ctx)
    }

    pub fn eraser(ctx: &Ctx) -> Self {
        Self::new_impl(ctx, true)
    }

    pub fn new(ctx: &Ctx) -> Self {
        Self::new_impl(ctx, false)
    }

    fn new_impl(ctx: &Ctx, eraser: bool) -> Self {
        Self {
            ctx: ctx.clone(),
            size: ctx.config.default_brush.size,
            eraser,
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

    fn draw_line(&self, texture: &mut Texture, p1: vec2<f32>, p2: vec2<f32>, color: Rgba<f32>) {
        let width = self.draw_width();
        let bb = {
            let bb = Aabb2::from_corners(p1, p2).extend_uniform(width);
            Aabb2 {
                min: bb.min.map(|x| x.floor() as i32),
                max: bb.max.map(|x| x.ceil() as i32),
            }
        };
        texture.draw(bb, |framebuffer, viewport| {
            let dir = (p2 - p1).normalize_or_zero();
            let normal = dir.rotate_90();
            let transform = mat3::translate((p1 + p2) / 2.0)
                * mat3::from_orts((p2 - p1) / 2.0, normal * width / 2.0);
            ugli::draw(
                framebuffer,
                &self.ctx.shaders.color_2d,
                ugli::DrawMode::TriangleFan,
                &*self.ctx.quad,
                ugli::uniforms! {
                    u_projection_matrix: mat3::ortho(bb.map(|x| x as f32)),
                    u_view_matrix: mat3::identity(),
                    u_transform: transform,
                    u_color: color,
                },
                ugli::DrawParameters {
                    viewport: Some(viewport),
                    ..default()
                },
            );
            for p in [p1, p2] {
                ugli::draw(
                    framebuffer,
                    &self.ctx.shaders.circle,
                    ugli::DrawMode::TriangleFan,
                    &*self.ctx.quad,
                    ugli::uniforms! {
                        u_projection_matrix: mat3::ortho(bb.map(|x| x as f32)),
                        u_view_matrix: mat3::identity(),
                        u_transform: mat3::translate(p) * mat3::scale_uniform(width / 2.0 + 0.5),
                        u_color: color,
                        u_radius: (width / 2.0) / (width / 2.0 + 0.5),
                    },
                    ugli::DrawParameters {
                        viewport: Some(viewport),
                        ..default()
                    },
                );
            }
        });
    }

    fn actual_color(&self, state: &State) -> Rgba<f32> {
        if self.eraser {
            Rgba::TRANSPARENT_BLACK
        } else {
            state.color
        }
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
        let color = self.actual_color(state);
        if let Some(idx) = state.selected {
            let plane = &mut state.model.planes[idx];
            if let Some(raycast) = plane.raycast(ray) {
                let pos = self.round_pos(raycast.texture_pos);
                self.draw_line(&mut plane.texture, pos, pos, color);
                return Some(BrushStroke {
                    prev_draw_pos: pos,
                    sfx: self.ctx.assets.scribble.play(),
                });
            }
        }
        None
    }
    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {
        let color = self.actual_color(state);
        if let Some(idx) = state.selected {
            let plane = &mut state.model.planes[idx];
            if let Some(raycast) = plane.raycast(ray) {
                let pos = self.round_pos(raycast.texture_pos);
                self.draw_line(&mut plane.texture, stroke.prev_draw_pos, pos, color);
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
                    texture: Texture::new(self.ctx.geng.ugli()),
                    transform: plane.transform,
                };

                if let Some(raycast) = preview_plane.raycast(ray) {
                    let pos = self.round_pos(raycast.texture_pos);
                    self.draw_line(
                        &mut preview_plane.texture,
                        pos,
                        pos,
                        if self.eraser {
                            Rgba::WHITE
                        } else {
                            state.color
                        },
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
                    preview_plane.transform *= mat4::translate(vec3(0.0, 0.0, offset));
                    if !self.eraser {
                        self.ctx
                            .draw_plane(&preview_plane, framebuffer, &state.camera);
                    }
                    self.ctx
                        .draw_plane_outline(&preview_plane, framebuffer, &state.camera);
                }
            }
        }

        let text = match self.eraser {
            false => "brush",
            true => "eraser",
        };
        let text = format!("{text} ({:.1} px)", self.size);
        let font = self.ctx.geng.default_font();
        let text_align = vec2::splat(geng::TextAlign::CENTER);
        let text_measure = font.measure(text.as_str(), text_align).unwrap();
        if !self.eraser {
            let color: Rgba<f32> = state.color;
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
