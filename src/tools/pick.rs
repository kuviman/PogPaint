use super::*;

pub struct Pick {}

impl Pick {
    pub fn new(ctx: &Ctx) -> Self {
        Self {}
    }
    fn find(&self, state: &State, ray: Ray) -> Option<usize> {
        let mut closest = None;
        for (idx, plane) in state.planes.iter().enumerate() {
            if let Some(raycast) = plane.raycast(ray) {
                if plane.texture.color_at(raycast.texture_pos).a == 0.0 {
                    continue;
                }
                let new = (raycast.t, idx);
                closest = match closest {
                    Some(current) => Some(std::cmp::min_by_key(current, new, |&(t, _idx)| r32(t))),
                    None => Some(new),
                };
            }
        }
        closest.map(|(_t, idx)| idx)
    }
}

impl Tool for Pick {
    type Stroke = ();
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<()> {
        state.selected = self.find(state, ray);
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
        if let Some(idx) = self.find(state, ray) {
            let plane = &state.planes[idx];
            plane
                .texture
                .draw_outline(framebuffer, &state.camera, plane.transform);
        }
    }
}
