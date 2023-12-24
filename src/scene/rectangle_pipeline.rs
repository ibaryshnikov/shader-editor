use iced_wgpu::wgpu::{
    self, Device, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SurfaceConfiguration,
};

pub struct PipelineData {
    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
}

use super::rectangle::TrianglePoint;

const VERTEX_SIZE: usize = std::mem::size_of::<TrianglePoint>();

impl PipelineData {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let shader_text = std::fs::read_to_string("shader.wgsl").expect("Should read the shader");

        Self::new_with_text(device, config, &shader_text)
    }

    pub fn new_with_text(device: &Device, config: &SurfaceConfiguration, text: &str) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(text)),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: VERTEX_SIZE as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }];

        let pipeline =
            Self::create_pipeline(device, config, &shader, &pipeline_layout, &vertex_buffers);
        PipelineData {
            pipeline_layout,
            pipeline,
        }
    }

    fn create_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &PipelineLayout,
        vertex_buffers: &[wgpu::VertexBufferLayout],
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
