use geng::prelude::*;

mod camera;
mod color;
mod ctx;
mod gizmo;
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

struct Transform {
    mode: gizmo::TransformMode,
    raw_transform: mat4<f32>,
}

pub struct State {
    ctx: Ctx,
    framebuffer_size: vec2<f32>,
    camera: Camera,
    ui_camera: Camera2d,
    brush_size: f32,
    color: Hsla<f32>,
    wheel: Option<Wheel>,
    planes: Vec<Plane>,
    selected: Option<usize>,
    prev_draw_pos: Option<vec2<f32>>,
    transform: Option<Transform>,
    scribble: Option<geng::SoundEffect>,
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
            planes: vec![Plane {
                texture: Texture::new(ctx),
                transform: mat4::identity(),
            }],
            selected: Some(0),
            prev_draw_pos: None,
            transform: None,
            scribble: None,
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

        for plane in &self.planes {
            plane.draw(framebuffer, &self.camera);
        }

        if let Some(idx) = self.selected {
            let transform =
                self.planes[idx].transform * mat4::scale_uniform(self.ctx.config.grid.cell_size);
            ugli::draw(
                framebuffer,
                &self.ctx.shaders.color_3d,
                ugli::DrawMode::Lines { line_width: 1.0 },
                &*self.ctx.grid,
                (
                    ugli::uniforms! {
                        u_transform: transform,
                        u_color: self.ctx.config.grid.color,
                    },
                    self.camera.uniforms(self.framebuffer_size),
                ),
                ugli::DrawParameters { ..default() },
            );
        }

        ugli::clear(framebuffer, None, Some(1.0), None);
        if self.ctx.geng.window().is_key_pressed(geng::Key::T) {
            if let Some(idx) = self.selected {
                let plane = &self.planes[idx];
                self.ctx
                    .gizmo
                    .draw(framebuffer, &self.camera, plane.transform);
            }
        }

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
        let mut timer = Timer::new();
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
                    } else if self.ctx.geng.window().is_key_pressed(geng::Key::T) {
                        if let Some(cursor_pos) = self.ctx.geng.window().cursor_position() {
                            if let Some(idx) = self.selected {
                                let plane = &self.planes[idx];
                                let ray = self
                                    .camera
                                    .pixel_ray(self.framebuffer_size, cursor_pos.map(|x| x as f32));
                                self.transform = Some(Transform {
                                    mode: self
                                        .ctx
                                        .gizmo
                                        .raycast(plane.transform, ray)
                                        .map(|v| (plane.transform * v.extend(0.0)).xyz()),
                                    raw_transform: plane.transform,
                                });
                                self.ctx.geng.window().lock_cursor();
                            }
                        }
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        if let Some(cursor_pos) = self.ctx.geng.window().cursor_position() {
                            let ray = self
                                .camera
                                .pixel_ray(self.framebuffer_size, cursor_pos.map(|x| x as f32));
                            if let Some(idx) = self.selected {
                                let plane = &mut self.planes[idx];
                                if let Some(pos) = plane.raycast(ray) {
                                    plane.texture.draw_line(
                                        pos.texture,
                                        pos.texture,
                                        self.brush_size,
                                        self.color.into(),
                                    );
                                    self.prev_draw_pos = Some(pos.texture);
                                    self.scribble = Some(self.ctx.assets.scribble.play());
                                }
                            }
                        }
                    }
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Left,
                } => {
                    if self.transform.is_some() {
                        self.ctx.geng.window().unlock_cursor();
                        self.transform = None;
                    }
                    if let Some(mut sfx) = self.scribble.take() {
                        sfx.stop();
                    }
                    self.prev_draw_pos = None;
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Middle,
                } => {
                    self.ctx.geng.window().lock_cursor();
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Middle,
                } => {
                    self.ctx.geng.window().unlock_cursor();
                }
                geng::Event::RawMouseMove { delta } => match &mut self.transform {
                    Some(transform) => {
                        match transform.mode {
                            gizmo::TransformMode::Translate(v) => {
                                transform.raw_transform = mat4::translate(
                                    v * vec3::dot(
                                        v,
                                        (self.camera.view_matrix().inverse()
                                            * delta.map(|x| x as f32).extend(0.0).extend(0.0))
                                        .xyz(),
                                    ),
                                ) * transform.raw_transform;
                            }
                            gizmo::TransformMode::Rotate(v) => {
                                let origin =
                                    (transform.raw_transform * vec3::ZERO.extend(1.0)).into_3d();
                                transform.raw_transform = mat4::translate(origin)
                                    * mat4::rotate(
                                        v,
                                        Angle::from_degrees(
                                            delta.y as f32 * self.ctx.config.camera.sensitivity,
                                        ),
                                    )
                                    * mat4::translate(-origin)
                                    * transform.raw_transform;
                            }
                        }
                        // TODO round
                        let plane = &mut self.planes[self.selected.unwrap()];
                        plane.transform = transform.raw_transform.map(|x| x.round());
                        let translation = plane.transform.col(3).xyz().map(|x| {
                            (x / self.ctx.config.grid.cell_size).round()
                                * self.ctx.config.grid.cell_size
                        });
                        plane.transform[(0, 3)] = translation.x;
                        plane.transform[(1, 3)] = translation.y;
                        plane.transform[(2, 3)] = translation.z;
                    }
                    None => {
                        self.camera.rot += Angle::from_degrees(
                            -delta.x as f32 * self.ctx.config.camera.sensitivity,
                        );
                        self.camera.attack = (self.camera.attack
                            + Angle::from_degrees(
                                -delta.y as f32 * self.ctx.config.camera.sensitivity,
                            ))
                        .clamp_abs(Angle::from_degrees(90.0));
                    }
                },
                geng::Event::CursorMove { position } => {
                    let ray = self
                        .camera
                        .pixel_ray(self.framebuffer_size, position.map(|x| x as f32));
                    let mut drawn = false;
                    if let Some(idx) = self.selected {
                        let plane = &mut self.planes[idx];
                        if let Some(pos) = plane.raycast(ray) {
                            if let Some(prev) = self.prev_draw_pos {
                                plane.texture.draw_line(
                                    prev,
                                    pos.texture,
                                    self.brush_size,
                                    self.color.into(),
                                );
                                self.prev_draw_pos = Some(pos.texture);
                                drawn = true;
                            }
                        }
                    }
                    if !drawn {
                        self.prev_draw_pos = None;
                    }
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Right,
                } => {
                    self.wheel = None;
                }
                geng::Event::Wheel { delta } => {
                    self.camera.distance *= self.ctx.config.camera.zoom_speed.powf(-delta as f32);
                }
                geng::Event::Draw => {
                    let delta_time = timer.tick();
                    let mut mov = vec3::<i32>::ZERO;
                    if self.ctx.geng.window().is_key_pressed(geng::Key::W) {
                        mov.y += 1;
                    }
                    if self.ctx.geng.window().is_key_pressed(geng::Key::A) {
                        mov.x -= 1;
                    }
                    if self.ctx.geng.window().is_key_pressed(geng::Key::S) {
                        mov.y -= 1;
                    }
                    if self.ctx.geng.window().is_key_pressed(geng::Key::D) {
                        mov.x += 1;
                    }
                    if self.ctx.geng.window().is_key_pressed(geng::Key::Space) {
                        mov.z += 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(geng::Key::ControlLeft)
                    {
                        mov.z -= 1;
                    }
                    let mov = mov
                        .xy()
                        .map(|x| x as f32)
                        .rotate(self.camera.rot)
                        .extend(mov.z as f32);
                    self.camera.pos += mov
                        * delta_time.as_secs_f64() as f32
                        * self.ctx.config.camera.move_speed
                        * self.camera.distance;

                    self.ctx
                        .geng
                        .clone()
                        .window()
                        .with_framebuffer(|framebuffer| self.draw(framebuffer));
                }
                geng::Event::KeyPress {
                    key: geng::Key::Tab,
                } => {
                    if self.planes.is_empty() {
                        self.selected = None;
                    } else {
                        self.selected =
                            Some(self.selected.map_or(0, |idx| (idx + 1) % self.planes.len()));
                    }
                }
                geng::Event::KeyPress { key: geng::Key::C } => {
                    self.planes.push(Plane {
                        texture: Texture::new(&self.ctx),
                        transform: mat4::translate(self.camera.pos),
                    });
                    self.selected = Some(self.planes.len() - 1);
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
