use super::*;

pub struct Pick {}

impl Pick {
    pub fn new(ctx: &Ctx) -> Self {
        Self {}
    }
}

impl Tool for Pick {
    type Stroke = ();
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<()> {
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
        state.selected = closest.map(|(_t, idx)| idx);
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
