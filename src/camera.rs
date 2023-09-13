use super::*;

pub struct Camera {
    pub pos: vec3<f32>,
    pub attack: Angle<f32>,
    pub rot: Angle<f32>,
    pub distance: f32,
    pub fov: Angle<f32>,
}

impl AbstractCamera3d for Camera {
    fn view_matrix(&self) -> mat4<f32> {
        mat4::translate(vec3(0.0, 0.0, -self.distance))
            * mat4::rotate_x(self.attack - Angle::from_degrees(90.0))
            * mat4::rotate_z(-self.rot)
            * mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: vec2<f32>) -> mat4<f32> {
        mat4::perspective(
            self.fov.as_radians(),
            framebuffer_size.aspect(),
            0.1,
            1000.0,
        )
    }
}
