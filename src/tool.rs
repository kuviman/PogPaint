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
        stroke: Option<&mut Self::Stroke>,
        state: &mut State,
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
        stroke: Option<&mut AnyStroke>,
        state: &mut State,
    ) {
        self.inner
            .draw(framebuffer, stroke.map(|stroke| &mut *stroke.inner), state)
    }
}

trait DynTool {
    fn start(&mut self, state: &mut State, ray: Ray) -> Option<Box<dyn Any>>;
    fn resume(&mut self, stroke: &mut dyn Any, state: &mut State, ray: Ray);
    fn end(&mut self, stroke: Box<dyn Any>, state: &mut State, ray: Ray);
    fn draw(
        &mut self,
        framebuffer: &mut ugli::Framebuffer,
        stroke: Option<&mut dyn Any>,
        state: &mut State,
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
        stroke: Option<&mut dyn Any>,
        state: &mut State,
    ) {
        <T as Tool>::draw(
            self,
            framebuffer,
            stroke.map(|stroke| stroke.downcast_mut().unwrap()),
            state,
        );
    }
}
