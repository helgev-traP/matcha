pub struct Vertex {
    pub position: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct RectangleDescriptor {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub radius: f32,
    pub div: u16,
}

impl RectangleDescriptor {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            radius: 0.0,
            div: 0,
        }
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.min(self.width / 2.0).min(self.height / 2.0);
        self.div = 16.min(radius as u16).max(1);
        self
    }

    pub fn division(mut self, div: u16) -> Self {
        self.div = div.max(1);
        self
    }
}

impl Default for RectangleDescriptor {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            radius: 0.0,
            div: 1,
        }
    }
}

pub fn rectangle(desc: RectangleDescriptor) -> (Vec<Vertex>, Vec<u16>) {
    if desc.width == 0.0 || desc.height == 0.0 {
        (vec![], vec![])
    } else if desc.radius == 0.0 {
        // 4 vertices
        let vertices = vec![
            Vertex {
                position: [desc.x, -desc.y, 0.0],
            },
            Vertex {
                position: [desc.x, -desc.y - desc.height, 0.0],
            },
            Vertex {
                position: [desc.x + desc.width, -desc.y - desc.height, 0.0],
            },
            Vertex {
                position: [desc.x + desc.width, -desc.y, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    } else {
        let mut vertex = Vec::with_capacity(desc.div as usize * 4 + 4);
        let mut indices = Vec::with_capacity(desc.div as usize * 12);

        // arrangement of vertices
        //
        // round A ---- round D
        //       |            |
        //       |            |
        //       |            |
        // round B ---- round C

        // Vertices

        // A

        for i in 0..=desc.div {
            let angle = std::f32::consts::PI / 2.0 * i as f32 / desc.div as f32;
            let x = desc.x + desc.radius * (1.0 - angle.sin());
            let y = -desc.y - desc.radius * (1.0 - angle.cos());
            vertex.push(Vertex {
                position: [x, y, 0.0],
            });
        }

        // B

        for i in 0..=desc.div {
            let angle = std::f32::consts::PI / 2.0 * i as f32 / desc.div as f32;
            let x = desc.x + desc.radius * (1.0 - angle.cos());
            let y = -desc.y - desc.height + desc.radius * (1.0 - angle.sin());
            vertex.push(Vertex {
                position: [x, y, 0.0],
            });
        }

        // C

        for i in 0..=desc.div {
            let angle = std::f32::consts::PI / 2.0 * i as f32 / desc.div as f32;
            let x = desc.x + desc.width - desc.radius * (1.0 - angle.sin());
            let y = -desc.y - desc.height + desc.radius * (1.0 - angle.cos());
            vertex.push(Vertex {
                position: [x, y, 0.0],
            });
        }

        // D

        for i in 0..=desc.div {
            let angle = std::f32::consts::PI / 2.0 * i as f32 / desc.div as f32;
            let x = desc.x + desc.width - desc.radius * (1.0 - angle.cos());
            let y = -desc.y - desc.radius * (1.0 - angle.sin());
            vertex.push(Vertex {
                position: [x, y, 0.0],
            });
        }

        // Indices

        for i in 0..desc.div * 2 {
            indices.push(0);
            indices.push(i + 1);
            indices.push(i + 2);

            indices.push(desc.div * 2 + 2);
            indices.push(desc.div * 2 + 2 + i + 1);
            indices.push(desc.div * 2 + 2 + i + 2);
        }

        indices.push(0);
        indices.push(desc.div * 2 + 1);
        indices.push(desc.div * 2 + 2);

        indices.push(desc.div * 2 + 2);
        indices.push(desc.div * 4 + 3);
        indices.push(0);

        (vertex, indices)
    }
}
