use geng::prelude::*;

mod camera;
mod color;
mod ctx;
mod plane;
mod texture;
mod wheel;

use camera::Camera;
use ctx::*;
use plane::Plane;
use texture::Texture;
use wheel::*;

#[derive(clap::Parser)]
struct Cli {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

pub struct State {
    ctx: Ctx,
    framebuffer_size: vec2<f32>,
    camera: Camera,
    ui_camera: Camera2d,
    brush_size: f32,
    color: Hsla<f32>,
    wheel: Option<Wheel>,
    plane: Plane,
    prev_draw_pos: Option<vec2<f32>>,
}

impl State {
    pub async fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            framebuffer_size: vec2::splat(1.0),
            camera: Camera {
                pos: vec3::ZERO,
                rot: Angle::from_degrees(ctx.config.camera.rotation),
                fov: Angle::from_degrees(ctx.config.camera.fov),
                attack: Angle::from_degrees(ctx.config.camera.attack),
                distance: ctx.config.camera.distance,
            },
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: ctx.config.ui.fov,
            },
            brush_size: ctx.config.default_brush_size,
            color: Hsla::new(0.0, 1.0, 0.5, 1.0),
            wheel: None,
            plane: Plane {
                texture: Texture::new(ctx),
                transform: mat4::identity(),
            },
            prev_draw_pos: None,
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
        self.plane.draw(framebuffer, &self.camera);
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
            color::handle_event(&mut self, &event);
            match event {
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
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        if let Some(cursor_pos) = self.ctx.geng.window().cursor_position() {
                            let ray = self
                                .camera
                                .pixel_ray(self.framebuffer_size, cursor_pos.map(|x| x as f32));
                            if let Some(pos) = self.plane.raycast(ray) {
                                self.plane.texture.draw_line(
                                    pos.texture,
                                    pos.texture,
                                    self.brush_size,
                                    self.color.into(),
                                );
                                self.prev_draw_pos = Some(pos.texture);
                            }
                        }
                    }
                }
                geng::Event::CursorMove { position } => {
                    let ray = self
                        .camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32));
                    if let Some(pos) = self.plane.raycast(ray) {
                        if let Some(prev) = self.prev_draw_pos {
                            self.plane.texture.draw_line(
                                prev,
                                pos.texture,
                                self.brush_size,
                                self.color.into(),
                            );
                            self.prev_draw_pos = Some(pos.texture);
                        }
                    } else {
                        self.prev_draw_pos = None;
                    }
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Left,
                } => {
                    self.prev_draw_pos = None;
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
