#![allow(dead_code, unused_variables)]

use geng::prelude::*;

use pog_paint::*;

mod camera;
mod color;
mod config;
mod ctx;
mod gizmo;
mod keybind;
mod keys;
mod palette;
mod save;
mod tool;
mod tools;
mod wheel;

use camera::Camera;
use config::Config;
use ctx::*;
use keybind::KeyBind;
use palette::Palette;
use tool::*;
use wheel::*;

#[derive(clap::Parser)]
struct Cli {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

pub struct State {
    color: Rgba<f32>,
    camera: Camera,
    selected: Option<usize>,
    model: Model,
}

impl State {
    pub fn new(ctx: &Ctx) -> Self {
        let mut model = Model::new(ctx.geng.ugli());
        model.planes.push(Plane {
            texture: Texture::new(ctx.geng.ugli()),
            transform: mat4::identity(),
        });
        Self {
            camera: Camera {
                pos: vec3::ZERO,
                rot: Angle::from_degrees(ctx.config.camera.rotation),
                fov: Angle::from_degrees(ctx.config.camera.fov),
                attack: Angle::from_degrees(ctx.config.camera.attack),
                distance: ctx.config.camera.distance,
            },
            selected: Some(0),
            model,
            color: ctx.config.default_brush.color,
        }
    }
}

struct TempTool {
    tool: AnyTool,
    cancel_on: Option<geng::Event>,
}

struct Toolbelt {
    primary: AnyTool,
    temp: Option<TempTool>,
}

impl Toolbelt {
    fn current(&mut self) -> &mut AnyTool {
        if let Some(temp) = &mut self.temp {
            return &mut temp.tool;
        }
        &mut self.primary
    }
}

pub struct App {
    ctx: Ctx,
    wheel: Option<Wheel>,
    color_chooser: Option<color::Chooser>,
    ui_camera: Camera2d,
    framebuffer_size: vec2<f32>,
    toolbelt: Toolbelt,
    state: State,
    drag_start: Option<vec3<f32>>,
    load_sender: std::sync::mpsc::Sender<Model>,
    load_recv: std::sync::mpsc::Receiver<Model>,
    history: Vec<Model>,
    history_pos: usize,
}

impl App {
    pub async fn new(ctx: &Ctx) -> Self {
        let (load_sender, load_recv) = std::sync::mpsc::channel();
        Self {
            ctx: ctx.clone(),
            framebuffer_size: vec2::splat(1.0),
            ui_camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: ctx.config.ui.fov,
            },
            toolbelt: Toolbelt {
                primary: AnyTool::new(tools::Brush::default(ctx)),
                temp: None,
            },
            wheel: None,
            state: State::new(ctx),
            drag_start: None,
            load_sender,
            load_recv,
            color_chooser: None,
            history: vec![],
            history_pos: 0,
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

        for plane in &self.state.model.planes {
            self.ctx.draw_plane(plane, framebuffer, &self.state.camera);
        }

        if let Some(idx) = self.state.selected {
            let plane = &self.state.model.planes[idx];
            self.ctx
                .draw_plane_outline(plane, framebuffer, &self.state.camera);
        }

        if let Some(idx) = self.state.selected {
            self.ctx.draw_grid(
                framebuffer,
                &self.state.camera,
                self.state.model.planes[idx].transform,
            );
        }

        let status_pos = self.ui_camera.fov / 2.0 - self.ctx.config.status.width / 2.0;
        let ray = self.ray(self.ctx.geng.window().cursor_position());
        self.toolbelt.current().draw(
            framebuffer,
            Some(ray),
            &mut self.state,
            &self.ui_camera,
            mat3::translate(vec2(
                0.0,
                match self.ctx.config.status.pos {
                    config::StatusPos::Top => status_pos,
                    config::StatusPos::Bottom => -status_pos,
                },
            )) * mat3::scale_uniform(self.ctx.config.status.width / 2.0),
        );

        ugli::clear(framebuffer, None, Some(1.0), None);

        if self
            .ctx
            .geng
            .window()
            .is_key_pressed(self.ctx.keys.show_origin)
        {
            self.ctx
                .gizmo
                .draw(framebuffer, &self.state.camera, mat4::identity());
        }

        if let Some(wheel) = &self.wheel {
            wheel::draw(self, wheel, framebuffer);
        }
        if let Some(chooser) = &mut self.color_chooser {
            chooser.draw(framebuffer);
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
            if let Ok(model) = self.load_recv.try_recv() {
                self.state.model = model;
            }

            let keys = self.ctx.keys.clone();

            if keys.color_chooser.matches(&event, &self.ctx) {
                if self.color_chooser.is_none() {
                    self.color_chooser =
                        Some(color::Chooser::new(&self.ctx, self.state.color.into()));
                } else {
                    self.color_chooser = None;
                }
            }

            if let Some(chooser) = &mut self.color_chooser {
                if let Some(color) = chooser.handle_event(&event, &mut self.state) {
                    self.state.color = color.into();
                }
                if !matches!(event, geng::Event::Draw) {
                    continue;
                }
            }

            if keys.undo.matches(&event, &self.ctx) {
                self.undo();
            }
            if keys.redo.matches(&event, &self.ctx) {
                self.redo();
            }

            // TODO color::handle_event(&mut self, &event);
            if let Some(temp) = &mut self.toolbelt.temp {
                if temp.cancel_on == Some(event.clone()) {
                    self.toolbelt.temp = None;
                    continue;
                }
            }
            self.toolbelt.current().handle_event(event.clone());

            if keys.save.matches(&event, &self.ctx) {
                self.save();
            }
            if keys.load.matches(&event, &self.ctx) {
                self.load();
            }

            if keys.delete_plane.matches(&event, &self.ctx) {
                if let Some(idx) = self.state.selected {
                    self.push_history();
                    self.state.model.planes.remove(idx);
                    self.state.selected = None;
                }
            }

            if keys.switch_plane.matches(&event, &self.ctx) {
                if self.state.model.planes.is_empty() {
                    self.state.selected = None;
                } else {
                    self.state.selected = Some(
                        self.state
                            .selected
                            .map_or(0, |idx| (idx + 1) % self.state.model.planes.len()),
                    );
                }
            }

            if keys.first_person.matches(&event, &self.ctx) {
                self.toggle_first_person();
            }

            if keys.palette.matches(&event, &self.ctx) {
                Palette::start(&mut self);
            }

            // https://stackoverflow.com/questions/31403723/how-to-declare-a-higher-ranked-lifetime-for-a-closure-argument
            fn constrain_tool<
                F: for<'a> Fn(&'a keys::ToolKeys) -> Option<(AnyTool, &'a KeyBind)>,
            >(
                f: F,
            ) -> F {
                f
            }
            let tool = constrain_tool(|keys: &keys::ToolKeys| -> Option<(AnyTool, &KeyBind)> {
                if let Some(bind) = &keys.brush {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::Brush::default(&self.ctx)), bind));
                    }
                }
                if let Some(bind) = &keys.eraser {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::Brush::eraser(&self.ctx)), bind));
                    }
                }
                if let Some(bind) = &keys.color_picker {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::ColorPicker::new(&self.ctx)), bind));
                    }
                }
                if let Some(bind) = &keys.transform {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::Transform::new(&self.ctx)), bind));
                    }
                }
                if let Some(bind) = &keys.pick {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::Pick::new(&self.ctx)), bind));
                    }
                }
                if let Some(bind) = &keys.create {
                    if bind.matches(&event, &self.ctx) {
                        return Some((AnyTool::new(tools::Create::new(&self.ctx)), bind));
                    }
                }
                None
            });

            if let Some((tool, bind)) = tool(&keys.tools.temp) {
                self.start_temp_tool(
                    tool,
                    Some(match bind.key {
                        keybind::KeyOrButton::Key(key) => geng::Event::KeyRelease { key },
                        keybind::KeyOrButton::Mouse(button) => geng::Event::MouseRelease { button },
                    }),
                );
            } else if let Some((tool, _bind)) = tool(&keys.tools.switch) {
                self.switch_primary_tool(tool);
            }

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
                                WheelType::Continuous(typ) => typ.select(hover, &mut self),
                            }
                        }
                    } else {
                        let ray = self.ray(self.ctx.geng.window().cursor_position());
                        self.push_history();
                        self.toolbelt.current().start(&mut self.state, ray);
                    }
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Left,
                } => {
                    let ray = self.ray(self.ctx.geng.window().cursor_position());
                    self.toolbelt.current().end(&mut self.state, ray);
                }
                geng::Event::MousePress {
                    button: geng::MouseButton::Middle,
                } => {
                    self.start_camera_look();
                }
                geng::Event::MouseRelease {
                    button: geng::MouseButton::Middle,
                } => {
                    self.stop_camera_look();
                }
                geng::Event::RawMouseMove { delta } => {
                    let old_drag_camera = self
                        .drag_start
                        .map(|p| self.state.camera.view_matrix() * p.extend(1.0));

                    #[cfg(target_arch = "wasm32")]
                    let delta = delta * 20.0;

                    self.state.camera.rot +=
                        Angle::from_degrees(-delta.x as f32 * self.ctx.config.camera.sensitivity);
                    self.state.camera.attack = (self.state.camera.attack
                        + Angle::from_degrees(
                            -delta.y as f32 * self.ctx.config.camera.sensitivity,
                        ))
                    .clamp_abs(Angle::from_degrees(90.0));

                    if let Some(old_camera) = old_drag_camera {
                        let drag = self.drag_start.unwrap();
                        let view_matrix = self.state.camera.view_matrix();
                        let p = view_matrix.inverse() * old_camera;
                        self.state.camera.pos -= p.into_3d() - drag;
                    }

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
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.forward)
                    {
                        mov.y += 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.left)
                    {
                        mov.x -= 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.back)
                    {
                        mov.y -= 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.right)
                    {
                        mov.x += 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.up)
                    {
                        mov.z += 1;
                    }
                    if self
                        .ctx
                        .geng
                        .window()
                        .is_key_pressed(self.ctx.keys.camera.down)
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
                geng::Event::KeyPress { key } if key == keys.camera.look => {
                    self.start_camera_look();
                }
                geng::Event::KeyRelease { key } if key == keys.camera.look => {
                    self.stop_camera_look();
                }
                _ => {}
            }
        }
    }

    fn switch_primary_tool(&mut self, tool: AnyTool) {
        if self.toolbelt.current().is_stroking() {
            return;
        }
        self.toolbelt.primary = tool;
    }

    fn start_temp_tool(&mut self, tool: AnyTool, cancel_on: Option<geng::Event>) {
        if self.toolbelt.current().is_stroking() {
            return;
        }
        self.toolbelt.temp = Some(TempTool { tool, cancel_on });
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
        self.toolbelt.current().resume(&mut self.state, ray);
    }

    fn ray(&self, cursor_position: Option<vec2<f64>>) -> Ray {
        self.state.camera.pixel_ray(
            self.framebuffer_size,
            cursor_position
                .map(|p| p.map(|x| x as f32))
                .unwrap_or(self.framebuffer_size / 2.0),
        )
    }

    fn start_camera_look(&mut self) {
        let ray = self.ray(self.ctx.geng.window().cursor_position());

        let mut closest = None::<f32>;
        for plane in &self.state.model.planes {
            if let Some(raycast) = plane.raycast(ray) {
                if plane.texture.color_at(raycast.texture_pos).a == 0.0 {
                    continue;
                }
                closest = match closest {
                    Some(current) => Some(current.min(raycast.t)),
                    None => Some(raycast.t),
                };
            }
        }
        if let Some(t) = closest {
            self.drag_start = Some(ray.from + ray.dir * t);
        }

        if self.state.camera.distance != 0.0 {
            self.ctx.geng.window().lock_cursor();
        }
    }

    fn stop_camera_look(&mut self) {
        if self.state.camera.distance != 0.0 {
            self.ctx.geng.window().unlock_cursor();
        }
        self.drag_start = None;
    }

    fn push_history(&mut self) {
        self.history.truncate(self.history_pos);
        self.history.push(self.state.model.clone());
        self.history_pos += 1;
    }
    fn undo(&mut self) {
        if self.history_pos >= self.history.len() {
            self.push_history();
            self.history_pos -= 1;
        }
        if self.history_pos != 0 {
            self.history_pos -= 1;
            self.state.model = self.history[self.history_pos].clone();
        }
        self.fix_selected();
    }
    fn redo(&mut self) {
        if self.history_pos + 1 < self.history.len() {
            self.history_pos += 1;
            self.state.model = self.history[self.history_pos].clone();
        }
        self.fix_selected();
    }
    fn fix_selected(&mut self) {
        if let Some(idx) = self.state.selected {
            if self.state.model.planes.get(idx).is_none() {
                self.state.selected = None;
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
            App::new(&Ctx::new(&geng).await).await.run().await;
        },
    );
}
