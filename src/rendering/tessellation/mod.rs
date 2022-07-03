use lyon::math::{point, Point};
use lyon::path::{PathBuffer};
use lyon::path::builder::PathBuilder;
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};
use lyon::tessellation::geometry_builder::simple_builder;
use crate::rendering::deformed_rendering::Vertex;

pub struct PathTessellator {
    path_buffer: PathBuffer,
    buffers: VertexBuffers<Point, u16>
}

impl PathTessellator {
    pub fn new() -> Self {
        Self {
            path_buffer: PathBuffer::new(),
            buffers: VertexBuffers::new()
        }
    }

    pub fn tessellate_polyline_fill(
        &mut self,
        vertices_to_extend: &mut Vec<Vertex>,
        indices_to_extend: &mut Vec<u16>,
        positions: &[(i16, i16)]
    ) {
        if positions.len() <= 1 {
            return;
        }

        self.path_buffer.clear();

        let mut builder = self.path_buffer.builder();

        builder.begin(point(positions[0].0 as f32 + 0.5, positions[0].1 as f32 + 0.5));
        for ix in 0..positions.len() {
            let pos = positions[(ix + 1) % positions.len()];
            builder.line_to(point(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5));
        }
        builder.close();
        let path_id = builder.build();

        self.buffers.vertices.clear();
        self.buffers.indices.clear();

        let mut tessellator = FillTessellator::new();
        {
            let mut geometry_builder = simple_builder(&mut self.buffers);
            tessellator.tessellate_path(
                self.path_buffer.get(path_id),
                &FillOptions::default(),
                &mut geometry_builder
            ).unwrap();
        }

        for vertex in self.buffers.vertices.iter() {
            vertices_to_extend.push(Vertex { position: (vertex.x as i16, vertex.y as i16) })
        }
        indices_to_extend.extend(self.buffers.indices.iter());
    }
}