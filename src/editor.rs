use iced_wgpu::wgpu::{self, Device, SurfaceConfiguration};

use crate::scene::{rectangle_pipeline, Scene};

pub struct Editor {
    scene: Scene,
    rectangle_pipeline_data: rectangle_pipeline::PipelineData,
}

impl Editor {
    pub fn init(config: &SurfaceConfiguration, device: &Device) -> Self {
        let rectangle_pipeline_data = rectangle_pipeline::PipelineData::new(device, config);
        let scene = Scene::new(device);
        Editor {
            scene,
            rectangle_pipeline_data,
        }
    }
    pub fn render(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.scene
            .render(&self.rectangle_pipeline_data, &mut render_pass);
    }

    pub fn update_rectangle_shader(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.rectangle_pipeline_data = rectangle_pipeline::PipelineData::new(device, config);
    }

    pub fn update_rectangle_shader_with_text(
        &mut self,
        device: &Device,
        config: &SurfaceConfiguration,
        text: &str,
    ) {
        self.rectangle_pipeline_data =
            rectangle_pipeline::PipelineData::new_with_text(device, config, text);
    }
}
