use super::*;

pub struct Plane {
    pub texture: Texture,
    pub transform: mat4<f32>,
}

pub struct Raycast {
    pub texture_pos: vec2<f32>,
    pub t: f32,
}

impl Plane {
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &impl AbstractCamera3d) {
        self.texture.draw(framebuffer, camera, self.transform);
    }

    pub fn raycast(&self, ray: geng::camera::Ray) -> Option<Raycast> {
        let inv_transform = self.transform.inverse();
        let local_ray = geng::camera::Ray {
            from: (inv_transform * ray.from.extend(1.0)).into_3d(),
            dir: (inv_transform * ray.dir.extend(0.0)).xyz(),
        };
        if local_ray.dir.z.approx_eq(&0.0) {
            return None;
        }
        let t = -local_ray.from.z / local_ray.dir.z;
        if t <= 0.0 {
            return None;
        }
        Some(Raycast {
            texture_pos: (local_ray.from + local_ray.dir * t).xy(),
            t,
        })
    }
}
