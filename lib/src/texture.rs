use super::*;

pub struct Texture {
    ugli: Ugli,
    pub texture: Option<ugli::Texture>,
    pub offset: vec2<i32>,
}

impl Texture {
    pub const MAX_SIZE: usize = 2048;

    pub fn new(ugli: &Ugli) -> Self {
        Self {
            ugli: ugli.clone(),
            texture: None,
            offset: vec2::ZERO,
        }
    }

    pub fn from(ugli: &Ugli, texture: Option<ugli::Texture>, offset: vec2<i32>) -> Self {
        Self {
            ugli: ugli.clone(),
            offset,
            texture,
        }
    }

    pub fn draw(&mut self, area: Aabb2<i32>, f: impl FnOnce(&mut ugli::Framebuffer, Aabb2<usize>)) {
        self.ensure_bounds(area);
        let Some(texture) = &mut self.texture else {
            return;
        };
        let mut framebuffer =
            ugli::Framebuffer::new_color(&self.ugli, ugli::ColorAttachment::Texture(texture));
        f(
            &mut framebuffer,
            area.map_bounds(|p| (p - self.offset).map(|x| x as usize)),
        );
    }

    pub fn bounding_box(&self) -> Option<Aabb2<i32>> {
        self.texture.as_ref().map(|texture| {
            Aabb2::point(self.offset).extend_positive(texture.size().map(|x| x as _))
        })
    }

    fn ensure_bounds(&mut self, bb: Aabb2<i32>) {
        let old_bb = self.bounding_box().unwrap_or(Aabb2::ZERO);
        let new_bb = Aabb2 {
            min: old_bb.min.zip(bb.min).map(|(a, b)| i32::min(a, b)),
            max: old_bb.max.zip(bb.max).map(|(a, b)| i32::max(a, b)),
        };
        if new_bb.width() as usize > Self::MAX_SIZE || new_bb.height() as usize > Self::MAX_SIZE {
            return;
        }
        if old_bb != new_bb {
            let mut new_texture =
                ugli::Texture::new_uninitialized(&self.ugli, new_bb.size().map(|x| x as usize));
            new_texture.set_filter(ugli::Filter::Nearest);
            {
                let mut framebuffer = ugli::Framebuffer::new_color(
                    &self.ugli,
                    ugli::ColorAttachment::Texture(&mut new_texture),
                );
                let framebuffer = &mut framebuffer;
                ugli::clear(framebuffer, Some(Rgba::TRANSPARENT_BLACK), None, None);
            }
            if let Some(texture) = &self.texture {
                let framebuffer = ugli::FramebufferRead::new_color(
                    &self.ugli,
                    ugli::ColorAttachmentRead::Texture(texture),
                );
                framebuffer.copy_to_texture(
                    &mut new_texture,
                    Aabb2::ZERO.extend_positive(framebuffer.size()),
                    (self.offset - new_bb.min).map(|x| x as usize),
                );
            }
            self.texture = Some(new_texture);
            self.offset = new_bb.min;
        }
    }

    pub fn color_at(&self, pos: vec2<f32>) -> Rgba<f32> {
        let Some(texture) = &self.texture else {
            return Rgba::TRANSPARENT_BLACK;
        };
        let pos = pos.map(|x| x.floor() as i32);
        let uv = pos - self.offset;
        if Aabb2::ZERO
            .extend_positive(texture.size().map(|x| x as _))
            .contains(uv)
        {
            let framebuffer = ugli::FramebufferRead::new_color(
                &self.ugli,
                ugli::ColorAttachmentRead::Texture(texture),
            );
            let data = framebuffer.read_color_at(
                Aabb2::point(uv.map(|x| x as usize)).extend_positive(vec2::splat(1)),
            );
            data.get(0, 0).convert()
        } else {
            Rgba::TRANSPARENT_BLACK
        }
    }
}
