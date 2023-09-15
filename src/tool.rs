use std::any::Any;

use super::*;

pub use geng::camera::Ray;

pub trait Tool: 'static {
    type Stroke: 'static;
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Self::Stroke>;
    fn resume(&mut self, stroke: &mut Self::Stroke, state: &mut State, ray: Ray);
    fn end(&mut self, stroke: Self::Stroke, state: &mut State, ray: Ray);
    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        stroke: Option<&mut Self::Stroke>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    );
}

pub struct AnyTool {
    inner: Box<dyn DynTool>,
}

pub struct AnyStroke {
    inner: Box<dyn Any>,
}

impl AnyTool {
    pub fn new(tool: impl Tool) -> Self {
        Self {
            inner: Box::new(tool),
        }
    }
    pub fn start(&mut self, state: &mut State, ray: Ray) -> Option<AnyStroke> {
        self.inner
            .start(state, ray)
            .map(|any| AnyStroke { inner: any })
    }
    pub fn resume(&mut self, stroke: &mut AnyStroke, state: &mut State, ray: Ray) {
        self.inner.resume(&mut *stroke.inner, state, ray)
    }
    pub fn end(&mut self, stroke: AnyStroke, state: &mut State, ray: Ray) {
        self.inner.end(stroke.inner, state, ray)
    }
    pub fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        stroke: Option<&mut AnyStroke>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        self.inner.draw(
            framebuffer,
            ray,
            stroke.map(|stroke| &mut *stroke.inner),
            state,
            ui_camera,
            status_pos,
        )
    }
}

trait DynTool {
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Box<dyn Any>>;
    fn resume(&mut self, stroke: &mut dyn Any, state: &mut State, ray: Ray);
    fn end(&mut self, stroke: Box<dyn Any>, state: &mut State, ray: Ray);
    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        stroke: Option<&mut dyn Any>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    );
}

impl<T: Tool> DynTool for T {
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Box<dyn Any>> {
        <T as Tool>::start(self, state, ray).map(|stroke| Box::new(stroke) as Box<dyn Any>)
    }
    fn resume(&mut self, stroke: &mut dyn Any, state: &mut State, ray: Ray) {
        <T as Tool>::resume(self, stroke.downcast_mut().unwrap(), state, ray);
    }
    fn end(&mut self, stroke: Box<dyn Any>, state: &mut State, ray: Ray) {
        <T as Tool>::end(self, *stroke.downcast().unwrap(), state, ray);
    }
    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        stroke: Option<&mut dyn Any>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        <T as Tool>::draw(
            self,
            framebuffer,
            ray,
            stroke.map(|stroke| stroke.downcast_mut().unwrap()),
            state,
            ui_camera,
            status_pos,
        );
    }
}
