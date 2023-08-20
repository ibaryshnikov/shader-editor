use iced_wgpu::wgpu::{self, Device, Queue, SurfaceConfiguration};

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

    pub fn resize(&self, _config: &SurfaceConfiguration, _queue: &Queue) {}

    pub fn render(&self, view: &wgpu::TextureView, device: &Device, queue: &Queue) {
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.scene.render(&self.rectangle_pipeline_data, &mut rpass);
        drop(rpass);

        queue.submit(Some(encoder.finish()));
        let future = device.pop_error_scope();
        tokio::spawn(async move {
            if let Some(error) = future.await {
                panic!("Rendering error: {}", error);
            }
        });
    }

    pub fn update_rectangle_shader(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.rectangle_pipeline_data = rectangle_pipeline::PipelineData::new(device, config);
    }
}
