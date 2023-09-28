use super::*;

pub struct Create {
    ctx: Ctx,
}

impl Create {
    pub fn new(ctx: &Ctx) -> Self {
        Self { ctx: ctx.clone() }
    }
    fn new_transform(&self, state: &State, ray: Ray) -> Option<mat4<f32>> {
        Some(self.ctx.round_matrix({
            let pos = match state.selected {
                Some(idx) => {
                    let plane = &state.model.planes[idx];
                    let Some(raycast) = plane.raycast(ray) else {
                        return None;
                    };
                    (plane.transform * raycast.texture_pos.extend(0.0).extend(1.0)).into_3d()
                }
                None => state.camera.pos,
            };
            let mut m = state.camera.view_matrix().inverse();
            m[(0, 3)] = pos.x;
            m[(1, 3)] = pos.y;
            m[(2, 3)] = pos.z;
            m
        }))
    }
}

impl Tool for Create {
    type Stroke = ();
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<()> {
        state.model.planes.push(Plane {
            texture: Texture::new(self.ctx.geng.ugli()),
            transform: self.new_transform(state, ray)?,
        });
        state.selected = Some(state.model.planes.len() - 1);
        None
    }
    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {}
    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray) {}

    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        _stroke: Option<&mut Self::Stroke>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        let Some(ray) = ray else { return };
        let Some(transform) = self.new_transform(state, ray) else {
            return;
        };
        self.ctx.draw_grid(
            framebuffer,
            &state.camera,
            transform * mat4::scale_uniform(1.0 / self.ctx.config.grid.cell_size),
        );
    }
}
