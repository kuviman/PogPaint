use super::*;

pub struct Renderer {
    shaders: Rc<Shaders>,
    quad: Rc<QuadData>,
    config: Rc<Config>,
    white: Rc<ugli::Texture>,
}

impl Renderer {
    pub fn new(
        _geng: &Geng,
        shaders: &Rc<Shaders>,
        quad: &Rc<QuadData>,
        config: &Rc<Config>,
        white: &Rc<ugli::Texture>,
    ) -> Self {
        Self {
            shaders: shaders.clone(),
            quad: quad.clone(),
            config: config.clone(),
            white: white.clone(),
        }
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl AbstractCamera3d,
        transform: mat4<f32>,
    ) {
        let transform = transform * mat4::scale_uniform(self.config.gizmo.size);
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let camera_forward =
            (camera.view_matrix() * transform).inverse() * vec4(0.0, 0.0, 1.0, 0.0);
        let camera_forward = camera_forward.xyz();
        let mut draw_axis = |color: Rgba<f32>, dir: vec3<f32>| {
            let mut draw_axis = |color, shrink: f32| {
                let transform = transform
                    * mat4::from_orts(
                        dir,
                        vec3::cross(dir, camera_forward).normalize_or_zero(),
                        camera_forward,
                    )
                    * mat4::scale(vec3(
                        0.5 - shrink / 2.0,
                        self.config.gizmo.width / 2.0 - shrink / 2.0,
                        1.0,
                    ))
                    * mat4::translate(vec3(1.0, 0.0, 0.0));
                ugli::draw(
                    framebuffer,
                    &self.shaders.texture,
                    ugli::DrawMode::TriangleFan,
                    &*self.quad,
                    (
                        ugli::uniforms! {
                            u_texture: &*self.white,
                            u_texture_size: self.white.size(),
                            u_transform: transform,
                            u_color: color,
                        },
                        camera.uniforms(framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::premultiplied_alpha()),
                        ..default()
                    },
                );
            };
            draw_axis(Rgba::BLACK, 0.0);
            draw_axis(color, self.config.gizmo.outline);
        };
        draw_axis(Rgba::RED, vec3::UNIT_X);
        draw_axis(Rgba::GREEN, vec3::UNIT_Y);
        draw_axis(Rgba::BLUE, vec3::UNIT_Z);

        let mut draw_ring = |color: Rgba<f32>, local_transform: mat4<f32>| {
            let mut draw_ring = |color: Rgba<f32>, shrink: f32| {
                ugli::draw(
                    framebuffer,
                    &self.shaders.ring,
                    ugli::DrawMode::TriangleFan,
                    &*self.quad,
                    (
                        ugli::uniforms! {
                            u_color: color,
                            u_outer_radius: 1.0 - shrink,
                            u_inner_radius: 1.0 - self.config.gizmo.width + shrink,
                            u_transform: transform * local_transform,
                        },
                        camera.uniforms(framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        depth_func: Some(ugli::DepthFunc::LessOrEqual),
                        ..default()
                    },
                );
            };
            draw_ring(Rgba::BLACK, 0.0);
            draw_ring(color, self.config.gizmo.outline);
        };
        draw_ring(
            Rgba::RED,
            mat4::from_orts(vec3::UNIT_Y, vec3::UNIT_Z, vec3::UNIT_X),
        );
        draw_ring(
            Rgba::GREEN,
            mat4::from_orts(vec3::UNIT_X, vec3::UNIT_Z, vec3::UNIT_Y),
        );
        draw_ring(
            Rgba::BLUE,
            mat4::from_orts(vec3::UNIT_X, vec3::UNIT_Y, vec3::UNIT_Z),
        );
    }

    pub fn raycast(&self, transform: mat4<f32>, ray: geng::camera::Ray) -> TransformMode {
        let distance_to_line = |dir: vec3<f32>| -> f32 {
            let line = geng::camera::Ray {
                from: (transform * vec3::ZERO.extend(1.0)).into_3d(),
                dir: (transform * dir.extend(0.0)).xyz(),
            };
            let normal = vec3::cross(line.dir, ray.dir);
            vec3::dot(normal.normalize_or_zero(), line.from - ray.from).abs()
        };
        let distance_to_ring = |local_transform: mat4<f32>| {
            let inv = (transform * local_transform).inverse();
            let ray = geng::camera::Ray {
                from: (inv * ray.from.extend(1.0)).into_3d(),
                dir: (inv * ray.dir.extend(0.0)).xyz(),
            };
            if ray.dir.z.approx_eq(&0.0) {
                return 1e9;
            }
            let t = -ray.from.z / ray.dir.z;
            let p = ray.from.xy() + ray.dir.xy() * t;
            (p.len() - self.config.gizmo.size).abs()
        };
        let to_line = |dir: vec3<f32>| (distance_to_line(dir), TransformMode::Translate(dir));
        let to_ring = |x: vec3<f32>, y: vec3<f32>, z: vec3<f32>| {
            let mat = mat4::from_orts(x, y, z);
            (distance_to_ring(mat), TransformMode::Rotate(z))
        };
        [
            to_line(vec3::UNIT_X),
            to_line(vec3::UNIT_Y),
            to_line(vec3::UNIT_Z),
            to_ring(vec3::UNIT_Y, vec3::UNIT_Z, vec3::UNIT_X),
            to_ring(vec3::UNIT_Z, vec3::UNIT_X, vec3::UNIT_Y),
            to_ring(vec3::UNIT_X, vec3::UNIT_Y, vec3::UNIT_Z),
        ]
        .into_iter()
        .min_by(|a, b| f32::total_cmp(&a.0, &b.0))
        .unwrap()
        .1
    }
}

#[derive(Copy, Clone)]
pub enum TransformMode {
    Translate(vec3<f32>),
    Rotate(vec3<f32>),
}
impl TransformMode {
    pub fn map(self, f: impl Fn(vec3<f32>) -> vec3<f32>) -> TransformMode {
        let mut result = self;
        let (Self::Translate(v) | Self::Rotate(v)) = &mut result;
        *v = f(*v);
        result
    }
}
