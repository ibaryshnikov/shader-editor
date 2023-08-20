use bytemuck::{Pod, Zeroable};
use iced_wgpu::wgpu::{self, Device, RenderPass};
use wgpu::util::DeviceExt;

use super::rectangle_pipeline::PipelineData;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TrianglePoint {
    pub position: [f32; 3],
}

fn vertex(x: f32, y: f32) -> TrianglePoint {
    TrianglePoint {
        position: [x, y, 0.0],
    }
}

pub struct Rectangle {
    pub vertices: Vec<TrianglePoint>,
    pub indices: Vec<u16>,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: usize,
}

impl Rectangle {
    pub fn new(device: &Device) -> Self {
        let vertex_data = vec![
            vertex(1.0, -1.0),
            vertex(-1.0, 1.0),
            vertex(-1.0, -1.0),
            vertex(1.0, 1.0),
        ];
        #[rustfmt::skip]
        let index_data = vec![
            0, 1, 2,
            0, 3, 1,
        ];

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer - Rectangle"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer - Rectangle"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = index_data.len();

        Rectangle {
            vertices: vertex_data,
            indices: index_data,
            vertex_buf,
            index_buf,
            index_count,
        }
    }

    pub fn render<'a>(&'a self, pipeline_data: &'a PipelineData, pass: &mut RenderPass<'a>) {
        pass.push_debug_group("Prepare data for Rectangle drawing");
        pass.set_pipeline(&pipeline_data.pipeline);
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.pop_debug_group();
        pass.insert_debug_marker("Drawing Rectangle");
        pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
    }
}
