use super::*;

pub struct Transform {
    ctx: Ctx,
    origin: Option<vec2<f32>>,
}

impl Transform {
    pub fn new(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            origin: None,
        }
    }
}

pub struct TransformStroke {
    mode: gizmo::TransformMode,
    start_point: vec3<f32>,
    original_transform: mat4<f32>,
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

impl Tool for Transform {
    type Stroke = TransformStroke;

    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Self::Stroke> {
        let Some(idx) = state.selected else {
            return None;
        };
        let plane = &state.planes[idx];
        match self.origin {
            None => {
                if let Some(raycast) = plane.raycast(ray) {
                    let pos = self.ctx.round_pos(raycast.texture_pos);
                    self.origin = Some(pos);
                }
            }
            Some(origin) => {
                let transform = plane.transform * mat4::translate(origin.extend(0.0));
                let mode = state
                    .ctx
                    .gizmo
                    .raycast(transform, ray)
                    .map(|v| (transform * v.extend(0.0)).xyz());
                let origin = (transform * vec4(0.0, 0.0, 0.0, 1.0)).into_3d();
                let start_point = match mode {
                    gizmo::TransformMode::Translate(axis) => closest_point_on_line(
                        ray,
                        Ray {
                            from: origin,
                            dir: axis,
                        },
                    ),
                    gizmo::TransformMode::Rotate(axis) => {
                        // dot(ray.from + ray.dir * t - plane_pos, axis) = 0
                        let t = vec3::dot(origin - ray.from, axis) / vec3::dot(ray.dir, axis);
                        ray.from + ray.dir * t
                    }
                };
                return Some(TransformStroke {
                    mode,
                    start_point,
                    original_transform: plane.transform,
                });
            }
        }
        None
    }

    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray) {
        let plane = &mut state.planes[state.selected.unwrap()];
        let origin =
            (stroke.original_transform * self.origin.unwrap().extend(0.0).extend(1.0)).into_3d();

        let normalized_original_transform = {
            let x = stroke.original_transform.col(0).xyz().normalize();
            let y = stroke.original_transform.col(1).xyz().normalize();
            let z = stroke.original_transform.col(2).xyz().normalize();
            mat4::from_orts(x, y, z)
        };

        let local_transform = match stroke.mode {
            gizmo::TransformMode::Translate(axis) => {
                let new_point = closest_point_on_line(
                    ray,
                    Ray {
                        from: origin,
                        dir: axis,
                    },
                );
                mat4::translate(new_point - stroke.start_point)
            }
            gizmo::TransformMode::Rotate(axis) => {
                let axis = axis.normalize_or_zero();
                let t = vec3::dot(origin - ray.from, axis) / vec3::dot(ray.dir, axis);
                let new_point = ray.from + ray.dir * t;
                let v1 = stroke.start_point - origin;
                let v2 = new_point - origin;
                let angle = vec2(vec3::dot(v1, v2), vec3::dot(vec3::cross(v1, v2), axis)).arg();
                mat4::rotate(axis, angle)
            }
        };

        plane.transform = mat4::translate(origin)
            * self
                .ctx
                .round_matrix(local_transform * normalized_original_transform)
            * mat4::translate(-self.origin.unwrap().extend(0.0));
    }

    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray) {}

    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        _ray: Option<Ray>,
        stroke: Option<&mut Self::Stroke>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        if let Some(origin) = self.origin {
            if let Some(idx) = state.selected {
                let plane = &state.planes[idx];
                state.ctx.gizmo.draw(
                    framebuffer,
                    &state.camera,
                    plane.transform * mat4::translate(origin.extend(0.0)),
                );
            }
        }

        state.ctx.geng.default_font().draw(
            framebuffer,
            ui_camera,
            "transform",
            vec2::splat(geng::TextAlign::CENTER),
            status_pos,
            Rgba::WHITE,
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::MousePress {
            button: geng::MouseButton::Right,
        } = event
        {
            self.origin = None;
        }
    }
}
