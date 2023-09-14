use super::*;

pub struct TransformTool {}

impl TransformTool {
    pub fn new(ctx: &Ctx) -> Self {
        Self {}
    }
}

pub struct TransformStroke {
    mode: gizmo::TransformMode,
    start_point: vec3<f32>,
    original_transform: mat4<f32>,
    raw_transform: mat4<f32>,
}

fn closest_point_to_line(ray: Ray, line: Ray) -> vec3<f32> {
    let closest_dir = vec3::cross(ray.dir, line.dir);
    let n = vec3::cross(line.dir, closest_dir);
    // dot(ray.from + ray.dir * t - line.from, n) = 0
    let t = vec3::dot(line.from - ray.from, n) / vec3::dot(ray.dir, n);
    ray.from + ray.dir * t
}

fn closest_point_on_line(ray: Ray, line: Ray) -> vec3<f32> {
    closest_point_to_line(line, ray)
}

impl Tool for TransformTool {
    type Stroke = TransformStroke;

    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Self::Stroke> {
        if let Some(idx) = state.selected {
            let plane = &state.planes[idx];
            let mode = state
                .ctx
                .gizmo
                .raycast(plane.transform, ray)
                .map(|v| (plane.transform * v.extend(0.0)).xyz());
            let plane_pos = (plane.transform * vec4(0.0, 0.0, 0.0, 1.0)).into_3d();
            let start_point = match mode {
                gizmo::TransformMode::Translate(axis) => closest_point_on_line(
                    ray,
                    Ray {
                        from: plane_pos,
                        dir: axis,
                    },
                ),
                gizmo::TransformMode::Rotate(axis) => {
                    // dot(ray.from + ray.dir * t - plane_pos, axis) = 0
                    let t = vec3::dot(plane_pos - ray.from, axis) / vec3::dot(ray.dir, axis);
                    ray.from + ray.dir * t
                }
            };
            return Some(TransformStroke {
                mode,
                start_point,
                original_transform: plane.transform,
                raw_transform: plane.transform,
            });
        }
        None
    }

    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {
        let plane = &mut state.planes[state.selected.unwrap()];
        let plane_pos = (stroke.original_transform * vec4(0.0, 0.0, 0.0, 1.0)).into_3d();
        stroke.raw_transform = match stroke.mode {
            gizmo::TransformMode::Translate(axis) => {
                let new_point = closest_point_on_line(
                    ray,
                    Ray {
                        from: plane_pos,
                        dir: axis,
                    },
                );
                mat4::translate(new_point - stroke.start_point) * stroke.original_transform
            }
            gizmo::TransformMode::Rotate(axis) => {
                let axis = axis.normalize_or_zero();
                let t = vec3::dot(plane_pos - ray.from, axis) / vec3::dot(ray.dir, axis);
                let new_point = ray.from + ray.dir * t;
                let v1 = stroke.start_point - plane_pos;
                let v2 = new_point - plane_pos;
                let angle = vec2(vec3::dot(v1, v2), vec3::dot(vec3::cross(v1, v2), axis)).arg();
                mat4::translate(plane_pos)
                    * mat4::rotate(axis, angle)
                    * mat4::translate(-plane_pos)
                    * stroke.original_transform
            }
        };
        plane.transform = stroke.raw_transform.map(|x| x.round());
        let translation = plane.transform.col(3).xyz().map(|x| {
            (x / state.ctx.config.grid.cell_size).round() * state.ctx.config.grid.cell_size
        });
        plane.transform[(0, 3)] = translation.x;
        plane.transform[(1, 3)] = translation.y;
        plane.transform[(2, 3)] = translation.z;
    }

    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray) {}

    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        stroke: Option<&mut Self::Stroke>,
        state: &mut State,
    ) {
        if let Some(idx) = state.selected {
            let plane = &state.planes[idx];
            state.ctx.gizmo.draw(
                framebuffer,
                &state.camera,
                stroke.map_or(plane.transform, |stroke| stroke.raw_transform),
            );
        }
    }
}
