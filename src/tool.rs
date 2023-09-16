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
    fn handle_event(&mut self, event: geng::Event) {
        #![allow(unused_variables)]
    }
}

pub struct AnyTool {
    inner: Box<dyn DynTool>,
    stroke: Option<Box<dyn Any>>,
}

impl AnyTool {
    pub fn new(tool: impl Tool) -> Self {
        Self {
            inner: Box::new(tool),
            stroke: None,
        }
    }
    pub fn is_stroking(&self) -> bool {
        self.stroke.is_some()
    }
    pub fn start(&mut self, state: &mut State, ray: Ray) {
        if self.stroke.is_some() {
            self.end(state, ray);
        }
        assert!(self.stroke.is_none());
        self.stroke = self.inner.start(state, ray);
    }
    pub fn resume(&mut self, state: &mut State, ray: Ray) {
        if let Some(stroke) = &mut self.stroke {
            self.inner.resume(&mut **stroke, state, ray)
        }
    }
    pub fn end(&mut self, state: &mut State, ray: Ray) {
        if let Some(stroke) = self.stroke.take() {
            self.inner.end(stroke, state, ray);
        }
    }
    pub fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        ray: Option<Ray>,
        state: &mut State,
        ui_camera: &dyn AbstractCamera2d,
        status_pos: mat3<f32>,
    ) {
        self.inner.draw(
            framebuffer,
            ray,
            self.stroke.as_mut().map(|stroke| &mut **stroke),
            state,
            ui_camera,
            status_pos,
        )
    }
    pub fn handle_event(&mut self, event: geng::Event) {
        self.inner.handle_event(event);
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
    fn handle_event(&mut self, event: geng::Event);
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
    fn handle_event(&mut self, event: geng::Event) {
        <T as Tool>::handle_event(self, event);
    }
}
