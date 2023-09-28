use super::*;

pub struct Shaders {
    pub hue_wheel: Rc<ugli::Program>,
    pub saturation_wheel: Rc<ugli::Program>,
    pub lightness_wheel: Rc<ugli::Program>,
    pub color_2d: Rc<ugli::Program>,
    pub color_3d: Rc<ugli::Program>,
    pub texture: Rc<ugli::Program>,
    pub circle: Rc<ugli::Program>,
    pub ring: Rc<ugli::Program>,
    pub outline: Rc<ugli::Program>,
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
                outline: Rc::new(
                    shader_lib
                        .compile(&manager.load::<String>(path.join("outline.glsl")).await?)?,
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
    pub keys: Rc<keys::Config>,
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
        let keys: Rc<keys::Config> = Rc::new(
            file::load_detect(run_dir().join("keys.toml"))
                .await
                .unwrap(),
        );
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
                keys,
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

    pub fn draw_grid(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl AbstractCamera3d,
        transform: mat4<f32>,
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let transform = transform * mat4::scale_uniform(self.config.grid.cell_size);
        ugli::draw(
            framebuffer,
            &self.shaders.color_3d,
            ugli::DrawMode::Lines { line_width: 1.0 },
            &*self.grid,
            (
                ugli::uniforms! {
                    u_transform: transform,
                    u_color: self.config.grid.color,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::LessOrEqual),
                ..default()
            },
        );
    }

    pub fn draw_plane(&self, plane: &Plane, framebuffer: &mut ugli::Framebuffer, camera: &Camera) {
        self.draw_plane_with(plane, framebuffer, camera, &self.shaders.texture, None);
    }

    pub fn draw_plane_outline(
        &self,
        plane: &Plane,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
    ) {
        self.draw_plane_with(
            plane,
            framebuffer,
            camera,
            &self.shaders.outline,
            Some(ugli::BlendMode {
                rgb: ugli::ChannelBlendMode {
                    src_factor: ugli::BlendFactor::OneMinusDstColor,
                    dst_factor: ugli::BlendFactor::Zero,
                    equation: ugli::BlendEquation::Add,
                },
                alpha: ugli::ChannelBlendMode {
                    src_factor: ugli::BlendFactor::One,
                    dst_factor: ugli::BlendFactor::Zero,
                    equation: ugli::BlendEquation::Add,
                },
            }),
        );
    }

    pub fn draw_plane_with(
        &self,
        plane: &Plane,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera,
        program: &ugli::Program,
        blend_mode: Option<ugli::BlendMode>,
    ) {
        let Some(texture) = &plane.texture.texture else {
            return;
        };
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let bb = plane.texture.bounding_box().unwrap().map(|x| x as f32);
        let transform = plane.transform
            * mat4::translate(bb.center().extend(0.0))
            * mat4::scale(bb.size().extend(1.0) / 2.0);
        ugli::draw(
            framebuffer,
            program,
            ugli::DrawMode::TriangleFan,
            &*self.quad,
            (
                ugli::uniforms! {
                 u_texture: texture,
                 u_texture_size: texture.size(),
                 u_transform: transform,
                 u_color: Rgba::WHITE,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::LessOrEqual),
                blend_mode,
                ..default()
            },
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Precision {
    Unbounded,
    Pixel,
    Grid,
}

impl Ctx {
    pub fn precision(&self) -> Precision {
        if self.geng.window().is_key_pressed(self.keys.precision.pixel) {
            Precision::Pixel
        } else if self
            .geng
            .window()
            .is_key_pressed(self.keys.precision.unbounded)
        {
            Precision::Unbounded
        } else {
            Precision::Grid
        }
    }
    pub fn round_pos(&self, pos: vec2<f32>) -> vec2<f32> {
        let precision = self.precision();
        let mut pos = pos;
        if precision >= Precision::Pixel {
            pos = pos.map(|x| x.round());
        }
        if precision >= Precision::Grid {
            pos =
                pos.map(|x| (x / self.config.grid.cell_size).round() * self.config.grid.cell_size);
        }
        pos
    }
    pub fn round_matrix(&self, m: mat4<f32>) -> mat4<f32> {
        let mut m = m;
        let precision = self.precision();
        if precision >= Precision::Pixel {
            m = m.map(|x| x.round());
        }
        if precision >= Precision::Grid {
            let translation = m
                .col(3)
                .xyz()
                .map(|x| (x / self.config.grid.cell_size).round() * self.config.grid.cell_size);
            m[(0, 3)] = translation.x;
            m[(1, 3)] = translation.y;
            m[(2, 3)] = translation.z;
        }
        m
    }
}
