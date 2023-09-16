use super::*;

pub struct Create {}

impl Create {
    pub fn new(ctx: &Ctx) -> Self {
        Self {}
    }
}

impl Tool for Create {
    type Stroke = ();
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<()> {
        state.planes.push(Plane {
            texture: Texture::new(&state.ctx),
            transform: state.ctx.round_matrix({
                let pos = match state.selected {
                    Some(idx) => {
                        let plane = &state.planes[idx];
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
            }),
        });
        state.selected = Some(state.planes.len() - 1);
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
    }
}
