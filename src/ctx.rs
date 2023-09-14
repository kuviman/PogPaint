use super::*;

#[derive(Deserialize)]
pub struct UiConfig {
    pub fov: f32,
}

#[derive(Deserialize)]
pub struct WheelConfig {
    pub size: f32,
    pub inner_radius: f32,
}

#[derive(Deserialize)]
pub struct CameraConfig {
    pub fov: f32,
    pub rotation: f32,
    pub attack: f32,
    pub distance: f32,
    pub sensitivity: f32,
    pub move_speed: f32,
    pub zoom_speed: f32,
}

#[derive(Deserialize)]
pub struct GizmoConfig {
    pub width: f32,
    pub outline: f32,
    pub size: f32,
}

#[derive(Deserialize)]
pub struct GridConfig {
    pub cell_size: f32,
    pub line_count: isize,
    pub color: Rgba<f32>,
}

#[derive(Deserialize)]
pub struct DefaultBrushConfig {
    pub size: f32,
    pub color: Rgba<f32>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub gizmo: GizmoConfig,
    pub camera: CameraConfig,
    pub default_brush: DefaultBrushConfig,
    pub background_color: Rgba<f32>,
    pub wheel: WheelConfig,
    pub ui: UiConfig,
    pub grid: GridConfig,
}

pub struct Shaders {
    pub hue_wheel: Rc<ugli::Program>,
    pub saturation_wheel: Rc<ugli::Program>,
    pub lightness_wheel: Rc<ugli::Program>,
    pub color_2d: Rc<ugli::Program>,
    pub color_3d: Rc<ugli::Program>,
    pub texture: Rc<ugli::Program>,
    pub circle: Rc<ugli::Program>,
    pub ring: Rc<ugli::Program>,
}

impl geng::asset::Load for Shaders {
    type Options = ();
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let mut shader_lib = geng::shader::Library::new(manager.ugli(), false, None);
            shader_lib.add(
                "color_wheel",
                &manager
                    .load::<String>(path.join("color_wheel.glsl"))
                    .await?,
            );
            Ok(Self {
                hue_wheel: Rc::new(
                    shader_lib
                        .compile(&manager.load::<String>(path.join("hue_wheel.glsl")).await?)?,
                ),
                saturation_wheel: Rc::new(
                    shader_lib.compile(
                        &manager
                            .load::<String>(path.join("saturation_wheel.glsl"))
                            .await?,
                    )?,
                ),
                lightness_wheel: Rc::new(
                    shader_lib.compile(
                        &manager
                            .load::<String>(path.join("lightness_wheel.glsl"))
                            .await?,
                    )?,
                ),
                color_2d: Rc::new(
                    shader_lib
                        .compile(&manager.load::<String>(path.join("color_2d.glsl")).await?)?,
                ),
                color_3d: Rc::new(
                    shader_lib
                        .compile(&manager.load::<String>(path.join("color_3d.glsl")).await?)?,
                ),
                circle: Rc::new(
                    shader_lib.compile(&manager.load::<String>(path.join("circle.glsl")).await?)?,
                ),
                texture: Rc::new(
                    shader_lib
                        .compile(&manager.load::<String>(path.join("texture.glsl")).await?)?,
                ),
                ring: Rc::new(
                    shader_lib.compile(&manager.load::<String>(path.join("ring.glsl")).await?)?,
                ),
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = None;
}

pub type QuadData = ugli::VertexBuffer<QuadVertex>;

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(options(looped = "true"))]
    pub scribble: geng::Sound,
}

pub struct CtxImpl {
    pub geng: Geng,
    pub config: Rc<Config>,
    pub shaders: Rc<Shaders>,
    pub quad: Rc<QuadData>,
    pub grid: Rc<QuadData>,
    pub gizmo: gizmo::Renderer,
    pub white: Rc<ugli::Texture>,
    pub assets: Rc<Assets>,
}

#[derive(Clone)]
pub struct Ctx {
    inner: Rc<CtxImpl>,
}

impl Deref for Ctx {
    type Target = CtxImpl;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(ugli::Vertex)]
pub struct QuadVertex {
    pub a_pos: vec2<f32>,
}

impl Ctx {
    pub async fn new(geng: &Geng) -> Self {
        let shaders: Rc<Shaders> = geng
            .asset_manager()
            .load(run_dir().join("shaders"))
            .await
            .unwrap();
        let quad = Rc::new(ugli::VertexBuffer::new_static(
            geng.ugli(),
            vec![
                QuadVertex {
                    a_pos: vec2(-1.0, -1.0),
                },
                QuadVertex {
                    a_pos: vec2(1.0, -1.0),
                },
                QuadVertex {
                    a_pos: vec2(1.0, 1.0),
                },
                QuadVertex {
                    a_pos: vec2(-1.0, 1.0),
                },
            ],
        ));
        let config: Rc<Config> = geng
            .asset_manager()
            .load(run_dir().join("config.toml"))
            .await
            .unwrap();
        let grid = Rc::new(ugli::VertexBuffer::new_static(geng.ugli(), {
            let mut vs = Vec::new();
            for x in -config.grid.line_count..=config.grid.line_count {
                vs.extend([
                    QuadVertex {
                        a_pos: vec2(x as f32, -config.grid.line_count as f32),
                    },
                    QuadVertex {
                        a_pos: vec2(x as f32, config.grid.line_count as f32),
                    },
                ]);
            }
            for y in -config.grid.line_count..=config.grid.line_count {
                vs.extend([
                    QuadVertex {
                        a_pos: vec2(-config.grid.line_count as f32, y as f32),
                    },
                    QuadVertex {
                        a_pos: vec2(config.grid.line_count as f32, y as f32),
                    },
                ]);
            }
            vs
        }));
        let white = Rc::new(ugli::Texture::new_with(geng.ugli(), vec2::splat(1), |_| {
            Rgba::WHITE
        }));
        let assets = Rc::new(
            geng.asset_manager()
                .load(run_dir().join("assets"))
                .await
                .unwrap(),
        );
        Self {
            inner: Rc::new(CtxImpl {
                gizmo: gizmo::Renderer::new(geng, &shaders, &quad, &config, &white),
                config,
                shaders,
                geng: geng.clone(),
                quad,
                white,
                grid,
                assets,
            }),
        }
    }
}
