use geng::prelude::*;

mod brush;
mod camera;
mod color;
mod ctx;
mod gizmo;
mod palette;
mod plane;
mod texture;
mod tool;
mod transform;
mod wheel;

use brush::Brush;
use camera::Camera;
use ctx::*;
use palette::Palette;
use plane::Plane;
use texture::Texture;
use tool::*;
use transform::TransformTool;
use wheel::*;

#[derive(clap::Parser)]
struct Cli {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

pub struct State {
    ctx: Ctx,
    camera: Camera,
    planes: Vec<Plane>,
    selected: Option<usize>,
    scribble: Option<geng::SoundEffect>,
}

impl State {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            camera: Camera {
                pos: vec3::ZERO,
                rot: Angle::from_degrees(ctx.config.camera.rotation),
                fov: Angle::from_degrees(ctx.config.camera.fov),
                attack: Angle::from_degrees(ctx.config.camera.attack),
                distance: ctx.config.camera.distance,
            },
            planes: vec![Plane {
                texture: Texture::new(ctx),
                transform: mat4::identity(),
            }],
            selected: Some(0),
            scribble: None,
        }
    }
    pub fn start_scribble(&mut self) {
        self.scribble = Some(self.ctx.assets.scribble.play());
    }
}

pub struct App {
    ctx: Ctx,
    wheel: Option<Wheel>,
    ui_camera: Camera2d,
    framebuffer_size: vec2<f32>,
    tool: AnyTool,
    stroke: Option<AnyStroke>,
    state: State,
}

impl App {
    pub async fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            framebuffer_size: vec2::splat(1.0),
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: ctx.config.ui.fov,
            },
            tool: AnyTool::new(Brush::default(ctx)),
            stroke: None,
            wheel: None,
            state: State::new(ctx),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            Some(self.ctx.config.background_color),
            Some(1.0),
            None,
        );

        for plane in &self.state.planes {
            plane.draw(framebuffer, &self.state.camera);
        }

        // Grid
        if let Some(idx) = self.state.selected {
            let transform = self.state.planes[idx].transform
                * mat4::scale_uniform(self.ctx.config.grid.cell_size);
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
                    self.state.camera.uniforms(self.framebuffer_size),
                ),
                ugli::DrawParameters { ..default() },
            );
        }

        self.tool
            .draw(framebuffer, self.stroke.as_mut(), &mut self.state);

        ugli::clear(framebuffer, None, Some(1.0), None);

        if let Some(wheel) = &self.wheel {
            wheel::draw(self, wheel, framebuffer);
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
            // TODO color::handle_event(&mut self, &event);
            match event {
                geng::Event::MousePress {
                    button: geng::MouseButton::Left,
                } => {
                    if let Some(wheel) = self.wheel.take() {
                        if let Some(hover) = wheel::calculate_hover(&self, &wheel) {
                            match wheel.typ {
                                WheelType::Items(typ) => {
                                    let hovered =
                                        wheel::items(&self, typ.item_count(), Some(hover))
                                            .into_iter()
                                            .position(|item| item.hovered);
                                    if let Some(hovered) = hovered {
                                        typ.select(hovered, &mut self);
                                    }
                                }
                                WheelType::Continious(typ) => typ.select(hover, &mut self),
                            }
                        }
                    } else {
                        let ray = self.ray(self.ctx.geng.window().cursor_position());
                        self.stroke = self.tool.start(&mut self.state, ray);
                    }
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Left,
                } => {
                    if let Some(stroke) = self.stroke.take() {
                        let ray = self.ray(self.ctx.geng.window().cursor_position());
                        self.tool.end(stroke, &mut self.state, ray);
                    }
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Middle,
                } if self.state.camera.distance != 0.0 => {
                    self.ctx.geng.window().lock_cursor();
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Middle,
                } if self.state.camera.distance != 0.0 => {
                    self.ctx.geng.window().unlock_cursor();
                }
                geng::Event::RawMouseMove { delta } => {
                    self.state.camera.rot +=
                        Angle::from_degrees(-delta.x as f32 * self.ctx.config.camera.sensitivity);
                    self.state.camera.attack = (self.state.camera.attack
                        + Angle::from_degrees(
                            -delta.y as f32 * self.ctx.config.camera.sensitivity,
                        ))
                    .clamp_abs(Angle::from_degrees(90.0));
                    self.handle_move(None);
                }
                geng::Event::CursorMove { position } => {
                    self.handle_move(Some(position));
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Right,
                } => {
                    self.wheel = None;
                }
                geng::Event::Wheel { delta } => {
                    self.state.camera.distance *=
                        self.ctx.config.camera.zoom_speed.powf(-delta as f32);
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
                        .rotate(self.state.camera.rot)
                        .extend(mov.z as f32);
                    self.state.camera.pos +=
                        mov * delta_time.as_secs_f64() as f32 * self.ctx.config.camera.move_speed;

                    self.ctx
                        .geng
                        .clone()
                        .window()
                        .with_framebuffer(|framebuffer| self.draw(framebuffer));
                }
                geng::Event::KeyPress {
                    key: geng::Key::Tab,
                } => {
                    if self.state.planes.is_empty() {
                        self.state.selected = None;
                    } else {
                        self.state.selected = Some(
                            self.state
                                .selected
                                .map_or(0, |idx| (idx + 1) % self.state.planes.len()),
                        );
                    }
                }
                geng::Event::KeyPress { key: geng::Key::C } => {
                    self.state.planes.push(Plane {
                        texture: Texture::new(&self.ctx),
                        transform: mat4::translate(self.state.camera.pos),
                    });
                    self.state.selected = Some(self.state.planes.len() - 1);
                }
                geng::Event::KeyPress { key: geng::Key::B } => {
                    self.switch_tool(Brush::default(&self.ctx));
                }
                geng::Event::KeyPress { key: geng::Key::E } => {
                    self.switch_tool(Brush::eraser(&self.ctx));
                }
                geng::Event::KeyPress { key: geng::Key::T } => {
                    self.switch_tool(TransformTool::new(&self.ctx));
                }
                geng::Event::KeyPress { key: geng::Key::F } => {
                    self.toggle_first_person();
                }
                geng::Event::KeyPress { key: geng::Key::Q } => Palette::start(&mut self),
                _ => {}
            }
        }
    }

    fn switch_tool(&mut self, tool: impl Tool) {
        if self.stroke.is_some() {
            return;
        }
        self.tool = AnyTool::new(tool);
    }

    fn toggle_first_person(&mut self) {
        let forward = (self.state.camera.view_matrix().inverse() * vec4(0.0, 0.0, -1.0, 0.0)).xyz();
        if self.state.camera.distance == 0.0 {
            self.state.camera.distance = self.ctx.config.camera.distance;
            self.state.camera.pos += forward * self.state.camera.distance;
            self.ctx.geng.window().unlock_cursor();
        } else {
            self.state.camera.pos -= forward * self.state.camera.distance;
            self.state.camera.distance = 0.0;
            self.ctx.geng.window().lock_cursor();
        }
    }

    fn handle_move(&mut self, cursor_position: Option<vec2<f64>>) {
        let ray = self.ray(cursor_position);
        if let Some(stroke) = &mut self.stroke {
            self.tool.resume(stroke, &mut self.state, ray);
        }
    }

    fn ray(&self, cursor_position: Option<vec2<f64>>) -> Ray {
        self.state.camera.pixel_ray(
            self.framebuffer_size,
            cursor_position
                .map(|p| p.map(|x| x as f32))
                .unwrap_or(self.framebuffer_size / 2.0),
        )
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
            App::new(&Ctx::new(&geng).await).await.run().await;
        },
    );
}
