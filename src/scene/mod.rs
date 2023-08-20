use iced_wgpu::wgpu::{Device, RenderPass};

pub mod rectangle;
pub mod rectangle_pipeline;

use rectangle::Rectangle;

pub struct Scene {
    rectangle: Rectangle,
}

impl Scene {
    pub fn new(device: &Device) -> Self {
        let rectangle = Rectangle::new(device);
        Scene { rectangle }
    }

    pub fn render<'a>(
        &'a self,
        rectangle_pipeline_data: &'a rectangle_pipeline::PipelineData,
        pass: &mut RenderPass<'a>,
    ) {
        self.rectangle.render(rectangle_pipeline_data, pass);
    }
}
