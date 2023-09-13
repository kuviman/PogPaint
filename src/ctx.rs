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

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub hue_wheel: ugli::Program,
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
