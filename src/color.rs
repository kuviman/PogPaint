use super::*;

enum Interact {
    Hue,
    Main,
    None,
}

pub struct Chooser {
    ctx: Ctx,
    interact: Interact,
    camera: Camera2d,
    color: Hsla<f32>,
    saturation_lightness_transform: mat3<f32>,
    hue_transform: mat3<f32>,
    current_color_transform: mat3<f32>,
}

impl Chooser {
    pub fn new(ctx: &Ctx, color: Hsla<f32>) -> Self {
        Self {
            ctx: ctx.clone(),
            interact: Interact::None,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 5.0,
            },
            color,
            saturation_lightness_transform: mat3::identity(),
            hue_transform: mat3::translate(vec2(0.0, 1.0)) * mat3::scale(vec2(1.0, 0.1)),
            current_color_transform: mat3::translate(vec2(0.0, 1.5)) * mat3::scale_uniform(0.1),
        }
    }
    pub fn handle_event(&mut self, event: &geng::Event, state: &mut State) -> Option<Hsla<f32>> {
        if *event
            == (geng::Event::MousePress {
                button: geng::MouseButton::Left,
            })
            || (matches!(event, geng::Event::CursorMove { .. })
                && self
                    .ctx
                    .geng
                    .window()
                    .is_button_pressed(geng::MouseButton::Left))
        {
            let pos = |mat: mat3<f32>| -> Option<vec2<f32>> {
                let cursor_pos = self.camera.screen_to_world(
                    self.ctx.geng.window().size().map(|x| x as f32),
                    self.ctx
                        .geng
                        .window()
                        .cursor_position()
                        .unwrap()
                        .map(|x| x as f32),
                );
                let p = (mat.inverse() * cursor_pos.extend(1.0)).into_2d();
                if Aabb2::ZERO.extend_uniform(1.0).contains(p) {
                    Some(p * 0.5 + vec2::splat(0.5))
                } else {
                    None
                }
            };

            if let Some(pos) = pos(self.saturation_lightness_transform) {
                self.color.s = pos.x;
                self.color.l = pos.y;
            }
            if let Some(pos) = pos(self.hue_transform) {
                self.color.h = pos.x;
            }
            return Some(self.color);
        }
        None
    }
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);

        ugli::draw(
            framebuffer,
            &self.ctx.shaders.saturation_value,
            ugli::DrawMode::TriangleFan,
            &*self.ctx.quad,
            (
                ugli::uniforms! {
                    u_transform: self.saturation_lightness_transform,
                    u_hue: self.color.h,
                },
                self.camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters::default(),
        );

        ugli::draw(
            framebuffer,
            &self.ctx.shaders.hue,
            ugli::DrawMode::TriangleFan,
            &*self.ctx.quad,
            (
                ugli::uniforms! {
                    u_transform: self.hue_transform,
                },
                self.camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters::default(),
        );

        self.ctx.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Quad::unit(self.color.into()).transform(self.current_color_transform),
        );
    }
}
