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

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
pub struct Config {
    pub background_color: Rgba<f32>,
    pub wheel: WheelConfig,
    pub ui: UiConfig,
}

pub struct Shaders {
    pub hue_wheel: Rc<ugli::Program>,
    pub saturation_wheel: Rc<ugli::Program>,
    pub lightness_wheel: Rc<ugli::Program>,
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
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = None;
}

pub struct CtxImpl {
    pub geng: Geng,
    pub config: Config,
    pub shaders: Shaders,
    pub quad: ugli::VertexBuffer<QuadVertex>,
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
        Self {
            inner: Rc::new(CtxImpl {
                config: geng
                    .asset_manager()
                    .load(run_dir().join("config.toml"))
                    .await
                    .unwrap(),
                shaders: geng
                    .asset_manager()
                    .load(run_dir().join("shaders"))
                    .await
                    .unwrap(),
                geng: geng.clone(),
                quad: ugli::VertexBuffer::new_static(
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
                ),
            }),
        }
    }
}
