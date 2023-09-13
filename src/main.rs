use geng::prelude::*;

mod ctx;
mod wheel;

use ctx::*;
use wheel::*;

#[derive(clap::Parser)]
struct Cli {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

struct HueWheel {
    ctx: Ctx,
    base: Hsla<f32>,
}

impl HueWheel {
    pub fn new(ctx: &Ctx, base: Hsla<f32>) -> Self {
        Self {
            ctx: ctx.clone(),
            base,
        }
    }
    fn color_at(&self, angle: Angle<f32>) -> Hsla<f32> {
        Hsla {
            h: {
                let x = angle.as_radians() / (2.0 * f32::PI);
                x - x.floor()
            },
            ..self.base
        }
    }
}

impl ContiniousWheel for HueWheel {
    fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn geng::AbstractCamera2d,
        transform: mat3<f32>,
        hover: Option<Angle<f32>>,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::draw(
            framebuffer,
            &self.ctx.shaders.hue_wheel,
            ugli::DrawMode::TriangleFan,
            &self.ctx.quad,
            (
                ugli::uniforms! {
                    u_transform: transform,
                    u_inner_radius: self.ctx.config.wheel.inner_radius,
                    u_actual_color: Rgba::from(hover.map_or(self.base, |angle| self.color_at(angle))),
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

struct State {
    ctx: Ctx,
    framebuffer_size: vec2<f32>,
    ui_camera: Camera2d,
    color: Hsla<f32>,
    wheel: Option<Wheel>,
}

impl State {
    pub async fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            framebuffer_size: vec2::splat(1.0),
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: ctx.config.ui.fov,
            },
            color: Hsla::new(0.0, 1.0, 0.5, 1.0),
            wheel: None,
        }
    }

    fn calculate_hover(&self, wheel: &Wheel) -> Option<Angle<f32>> {
        self.ctx.geng.window().cursor_position().and_then(|pos| {
            let pos = self
                .ui_camera
                .screen_to_world(self.framebuffer_size, pos.map(|x| x as f32));
            let delta = pos - wheel.pos;
            if delta.len() / self.ctx.config.wheel.size < self.ctx.config.wheel.inner_radius {
                return None;
            }
            Some(delta.arg())
        })
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            Some(self.ctx.config.background_color),
            Some(1.0),
            None,
        );
        if let Some(wheel) = &self.wheel {
            let hover = self.calculate_hover(wheel);
            let transform =
                mat3::translate(wheel.pos) * mat3::scale_uniform(self.ctx.config.wheel.size);
            match &wheel.typ {
                WheelType::Items => todo!(),
                WheelType::Continious(typ) => {
                    typ.draw(framebuffer, &self.ui_camera, transform, hover);
                }
            }
        }
    }

    fn start_wheel(&mut self, typ: WheelType) {
        self.wheel = Some(Wheel {
            pos: vec2::ZERO,
            typ,
        });
    }

    pub async fn run(mut self) {
        let mut events = self.ctx.geng.window().events();
        while let Some(event) = events.next().await {
            match event {
                geng::Event::KeyPress { key } => match key {
                    geng::Key::H => {
                        self.start_wheel(WheelType::Continious(Box::new(HueWheel::new(
                            &self.ctx, self.color,
                        ))));
                    }
                    _ => {}
                },
                geng::Event::MousePress {
                    button: geng::MouseButton::Left,
                } => {
                    if let Some(wheel) = self.wheel.take() {
                        if let Some(hover) = self.calculate_hover(&wheel) {
                            match wheel.typ {
                                WheelType::Items => todo!(),
                                WheelType::Continious(typ) => typ.select(hover, &mut self),
                            }
                        }
                    }
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Right,
                } => {
                    self.wheel = None;
                }
                geng::Event::Draw => {
                    self.ctx
                        .geng
                        .clone()
                        .window()
                        .with_framebuffer(|framebuffer| self.draw(framebuffer));
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let cli: Cli = clap::Parser::parse();
    geng::Geng::run_with(
        &{
            let mut options = geng::ContextOptions::default();
            options.window.title = "PogPaint".to_owned();
            options.with_cli(&cli.geng);
            options
        },
        |geng| async move {
            State::new(&Ctx::new(&geng).await).await.run().await;
        },
    );
}
