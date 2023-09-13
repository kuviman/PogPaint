use super::*;

struct ColorWheel {
    ctx: Ctx,
    program: Rc<ugli::Program>,
    base: Hsla<f32>,
    f: Box<dyn Fn(Hsla<f32>, f32) -> Hsla<f32>>,
}

impl ColorWheel {
    pub fn new(
        ctx: &Ctx,
        program: &Rc<ugli::Program>,
        base: Hsla<f32>,
        f: impl 'static + Fn(Hsla<f32>, f32) -> Hsla<f32>,
    ) -> Self {
        Self {
            ctx: ctx.clone(),
            program: program.clone(),
            base,
            f: Box::new(f),
        }
    }
    fn color_at(&self, angle: Angle<f32>) -> Hsla<f32> {
        let x = angle.as_radians() / (2.0 * f32::PI);
        let x = x - x.floor();
        (self.f)(self.base, x)
    }
}

impl ContiniousWheel for ColorWheel {
    fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn geng::AbstractCamera2d,
        transform: mat3<f32>,
        hover: Option<Angle<f32>>,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let actual_color = hover.map_or(self.base, |angle| self.color_at(angle));
        ugli::draw(
            framebuffer,
            &self.program,
            ugli::DrawMode::TriangleFan,
            &self.ctx.quad,
            (
                ugli::uniforms! {
                    u_transform: transform,
                    u_inner_radius: self.ctx.config.wheel.inner_radius,
                    u_actual_color: Rgba::from(actual_color),
                    u_actual_color_hsla: vec4(actual_color.h, actual_color.s, actual_color.l, actual_color.a),
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters { ..default() },
        );
    }

    fn select(&self, hover: Angle<f32>, state: &mut State) {
        state.color = self.color_at(hover);
    }
}

pub fn handle_event(state: &mut State, event: &geng::Event) {
    let ctx = &state.ctx;
    if let geng::Event::KeyPress { key } = event {
        match key {
            geng::Key::H => {
                state.start_wheel(WheelType::Continious(Box::new(ColorWheel::new(
                    ctx,
                    &ctx.shaders.hue_wheel,
                    state.color,
                    |color, h| Hsla { h, ..color },
                ))));
            }
            geng::Key::S => {
                state.start_wheel(WheelType::Continious(Box::new(ColorWheel::new(
                    ctx,
                    &ctx.shaders.saturation_wheel,
                    state.color,
                    |color, s| Hsla { s, ..color },
                ))));
            }
            geng::Key::L => {
                state.start_wheel(WheelType::Continious(Box::new(ColorWheel::new(
                    ctx,
                    &ctx.shaders.lightness_wheel,
                    state.color,
                    |color, l| Hsla { l, ..color },
                ))));
            }
            _ => {}
        }
    }
}
