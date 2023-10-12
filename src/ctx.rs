use std::path::PathBuf;

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
    pub saturation_value: Rc<ugli::Program>,
    pub hue: Rc<ugli::Program>,
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
                saturation_value: Rc::new(
                    shader_lib.compile(
                        &manager
                            .load::<String>(path.join("saturation_value.glsl"))
                            .await?,
                    )?,
                ),
                hue: Rc::new(
                    shader_lib.compile(&manager.load::<String>(path.join("hue.glsl")).await?)?,
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
    pub transparent_black: Rc<ugli::Texture>,
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

fn data_dir() -> PathBuf {
    run_dir().join("data")
}

impl Ctx {
    pub async fn new(geng: &Geng) -> Self {
        let shaders: Rc<Shaders> = geng
            .asset_manager()
            .load(data_dir().join("shaders"))
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
            .load(data_dir().join("config.toml"))
            .await
            .unwrap();
        let keys: Rc<keys::Config> = Rc::new(
            file::load_detect(data_dir().join("keys.toml"))
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
        let transparent_black =
            Rc::new(ugli::Texture::new_with(geng.ugli(), vec2::splat(1), |_| {
                Rgba::TRANSPARENT_BLACK
            }));
        let assets = Rc::new(
            geng.asset_manager()
                .load(data_dir().join("assets"))
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
                transparent_black,
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
        // let transform = plane.transform
        //     * mat4::translate(bb.center().extend(0.0))
        //     * mat4::scale(bb.size().extend(1.0) / 2.0);

        let empty_heightmap = Heightmap {
            texture: Texture::new(self.geng.ugli()),
            min: 0.0,
            max: 1.0,
        };
        let heightmap = plane.heightmap.as_ref().unwrap_or(&empty_heightmap);
        let heightmap_texture = heightmap
            .texture
            .texture
            .as_ref()
            .unwrap_or(&self.transparent_black);
        // from texture uv to heightmap uv
        let heightmap_uv_matrix = mat3::scale(heightmap_texture.size().map(|x| 1.0 / x as f32))
            * mat3::translate(
                (-heightmap.texture.offset + plane.texture.offset).map(|x| x as f32 + 0.5),
            )
            * mat3::scale(texture.size().map(|x| x as f32));

        #[derive(ugli::Vertex)]
        pub struct MeshVertex {
            pub a_pos: vec2<f32>,
            pub a_uv: vec2<f32>,
        }
        let mesh = ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
            let bb = plane.texture.bounding_box().unwrap();
            bb.points()
                .flat_map(|p| {
                    let quad = [vec2(0, 0), vec2(1, 0), vec2(1, 1), vec2(0, 1)].map(|v| p + v);
                    [quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]].map(|v| MeshVertex {
                        a_pos: v.map(|x| x as f32),
                        a_uv: (v - bb.bottom_left()).map(|x| x as f32)
                            / bb.size().map(|x| x as f32),
                    })
                })
                .collect()
        });
        ugli::draw(
            framebuffer,
            program,
            ugli::DrawMode::Triangles,
            &mesh,
            // &*self.quad,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_texture_size: texture.size(),
                    u_heightmap_texture: heightmap_texture,
                    u_heightmap_matrix: heightmap_uv_matrix,
                    u_min_height: heightmap.min,
                    u_max_height: heightmap.max,
                    u_transform: plane.transform,
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
