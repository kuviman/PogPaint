use super::*;

pub struct Palette {
    ctx: Ctx,
    colors: Vec<Rgba<f32>>,
}

impl Palette {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            colors: ctx.config.default_palette.clone(),
        }
    }
    pub fn start(app: &mut App) {
        app.start_wheel(WheelType::Items(Box::new(Self::new(&app.ctx))))
    }
}

impl ItemWheel for Palette {
    fn item_count(&self) -> usize {
        self.colors.len()
    }

    fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &dyn geng::AbstractCamera2d,
        transform: mat3<f32>,
        items: &[Item],
    ) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        for (item, color) in items.iter().zip(&self.colors) {
            let transform = transform * item.local_transform;
            ugli::draw(
                framebuffer,
                &self.ctx.shaders.color_2d,
                ugli::DrawMode::TriangleFan,
                &*self.ctx.quad,
                (
                    ugli::uniforms! {
                        u_transform: transform,
                        u_color: color,
                    },
                    camera.uniforms(framebuffer_size),
                ),
                ugli::DrawParameters { ..default() },
            );
        }
    }

    fn select(&self, item: usize, app: &mut App) {
        app.state.color = self.colors[item];
    }
}
