use super::*;

pub struct Texture {
    ctx: Ctx,
    texture: Option<ugli::Texture>,
    bb: Aabb2<i32>,
}

impl Texture {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            texture: None,
            bb: Aabb2::ZERO, // TODO
        }
    }
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &impl AbstractCamera2d) {
        let Some(texture) = &self.texture else { return };
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let bb = self.bb.map(|x| x as f32);
        let transform = mat3::translate(bb.center()) * mat3::scale(bb.size() / 2.0);
        ugli::draw(
            framebuffer,
            &self.ctx.shaders.texture,
            ugli::DrawMode::TriangleFan,
            &self.ctx.quad,
            (
                ugli::uniforms! {
                    u_texture: texture,
                    u_texture_size: texture.size(),
                    u_transform: transform,
                },
                camera.uniforms(framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::premultiplied_alpha()),
                ..default()
            },
        );
    }
    pub fn draw_line(&mut self, p1: vec2<f32>, p2: vec2<f32>, width: f32, color: Rgba<f32>) {
        self.ensure_bounds({
            let bb = Aabb2::from_corners(p1, p2).extend_uniform(width);
            Aabb2 {
                min: bb.min.map(|x| x.floor() as i32),
                max: bb.max.map(|x| x.ceil() as i32),
            }
        });
        let mut framebuffer = ugli::Framebuffer::new_color(
            self.ctx.geng.ugli(),
            ugli::ColorAttachment::Texture(self.texture.as_mut().unwrap()),
        );
        let framebuffer = &mut framebuffer;
        let normal = (p2 - p1).normalize_or_zero().rotate_90();
        let transform = mat3::translate((p1 + p2) / 2.0)
            * mat3::from_orts((p2 - p1) / 2.0, normal * width / 2.0);
        ugli::draw(
            framebuffer,
            &self.ctx.shaders.color,
            ugli::DrawMode::TriangleFan,
            &self.ctx.quad,
            ugli::uniforms! {
                u_projection_matrix: mat3::ortho(self.bb.map(|x| x as f32)),
                u_view_matrix: mat3::identity(),
                u_transform: transform,
                u_color: color,
            },
            ugli::DrawParameters { ..default() },
        );
    }
    fn ensure_bounds(&mut self, bb: Aabb2<i32>) {
        let new_bb = Aabb2 {
            min: self.bb.min.zip(bb.min).map(|(a, b)| i32::min(a, b)),
            max: self.bb.max.zip(bb.max).map(|(a, b)| i32::max(a, b)),
        };
        if self.bb != new_bb {
            let mut new_texture = ugli::Texture::new_uninitialized(
                self.ctx.geng.ugli(),
                new_bb.size().map(|x| x as usize),
            );
            {
                let mut framebuffer = ugli::Framebuffer::new_color(
                    self.ctx.geng.ugli(),
                    ugli::ColorAttachment::Texture(&mut new_texture),
                );
                let framebuffer = &mut framebuffer;
                ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), None, None);
            }
            if let Some(texture) = &self.texture {
                let framebuffer = ugli::FramebufferRead::new_color(
                    self.ctx.geng.ugli(),
                    ugli::ColorAttachmentRead::Texture(texture),
                );
                framebuffer.copy_to_texture(
                    &mut new_texture,
                    Aabb2::ZERO.extend_positive(framebuffer.size()),
                    (self.bb.min - new_bb.min).map(|x| x as usize),
                );
            }
            self.texture = Some(new_texture);
            self.bb = new_bb;
        }
    }
}
