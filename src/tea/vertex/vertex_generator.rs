pub struct Vertex {
    pub position: [f32; 3],
}

// box

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
            div: 1,
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
    // todo: check desc validity

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
        let vertex = make_rectangle_vertex(
            desc.x,
            desc.y,
            desc.width,
            desc.height,
            desc.radius,
            desc.div,
        );

        // Indices
        let mut indices = Vec::with_capacity(desc.div as usize * 12);

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

// border

/// # Border descriptor
///
/// border will be drawn **inside** the box
pub struct BorderDescriptor {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub radius: f32,
    pub div: u16,

    pub border_width: f32,
}

impl BorderDescriptor {
    pub fn new(width: f32, height: f32, border_width: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
            radius: 0.0,
            div: 0,
            border_width: border_width.min(width / 2.0).min(height / 2.0),
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

impl Default for BorderDescriptor {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            radius: 0.0,
            div: 1,
            border_width: 0.0,
        }
    }
}

pub fn border(desc: BorderDescriptor) -> (Vec<Vertex>, Vec<u16>) {
    // todo: check desc validity

    if desc.width == 0.0 || desc.height == 0.0 || desc.border_width == 0.0 {
        (vec![], vec![])
    } else if desc.radius == 0.0 {
        // 8 vertices
        let vertices = vec![
            // outer
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
            // inner
            Vertex {
                position: [desc.x + desc.border_width, -desc.y - desc.border_width, 0.0],
            },
            Vertex {
                position: [
                    desc.x + desc.border_width,
                    -desc.y - desc.height + desc.border_width,
                    0.0,
                ],
            },
            Vertex {
                position: [
                    desc.x + desc.width - desc.border_width,
                    -desc.y - desc.height + desc.border_width,
                    0.0,
                ],
            },
            Vertex {
                position: [
                    desc.x + desc.width - desc.border_width,
                    -desc.y - desc.border_width,
                    0.0,
                ],
            },
        ];

        let indices = vec![
            0, 4, 7, 7, 3, 0, // top
            3, 7, 6, 6, 2, 3, // right
            2, 6, 5, 5, 1, 2, // bottom
            1, 5, 4, 4, 0, 1, // left
        ];

        (vertices, indices)
    } else if desc.border_width < desc.radius {
        let mut vertex = make_rectangle_vertex(
            desc.x,
            desc.y,
            desc.width,
            desc.height,
            desc.radius,
            desc.div,
        );

        vertex.extend(make_rectangle_vertex(
            desc.x + desc.border_width,
            desc.y + desc.border_width,
            desc.width - desc.border_width * 2.0,
            desc.height - desc.border_width * 2.0,
            desc.radius - desc.border_width,
            desc.div,
        ));

        let mut indices = Vec::with_capacity(desc.div as usize * 24 + 24);
        let index_len = desc.div * 4 + 4;

        for i in 0..desc.div * 4 + 3 {
            indices.push(i);
            indices.push(i + 1);
            indices.push(index_len + i);

            indices.push(i + 1);
            indices.push(index_len + i + 1);
            indices.push(index_len + i);
        }

        indices.push(desc.div * 4 + 3);
        indices.push(0);
        indices.push(index_len + desc.div * 4 + 3);

        indices.push(0);
        indices.push(index_len);
        indices.push(index_len + desc.div * 4 + 3);

        (vertex, indices)
    } else if desc.border_width < desc.width / 2.0 && desc.border_width < desc.height / 2.0 {
        let mut vertex = make_rectangle_vertex(
            desc.x,
            desc.y,
            desc.width,
            desc.height,
            desc.radius,
            desc.div,
        );

        vertex.extend(vec![
            Vertex {
                position: [desc.x + desc.border_width, -desc.y - desc.border_width, 0.0],
            },
            Vertex {
                position: [
                    desc.x + desc.border_width,
                    -desc.y - desc.height + desc.border_width,
                    0.0,
                ],
            },
            Vertex {
                position: [
                    desc.x + desc.width - desc.border_width,
                    -desc.y - desc.height + desc.border_width,
                    0.0,
                ],
            },
            Vertex {
                position: [
                    desc.x + desc.width - desc.border_width,
                    -desc.y - desc.border_width,
                    0.0,
                ],
            },
        ]);

        let mut indices = Vec::with_capacity(desc.div as usize * 24 + 24);
        let index_len = desc.div * 4 + 4;

        // corners
        for i in 0..desc.div {
            // upper left
            indices.push(i);
            indices.push(i + 1);
            indices.push(index_len);
            // lower left
            indices.push(desc.div + 1 + i);
            indices.push(desc.div + 1 + i + 1);
            indices.push(index_len + 1);
            // lower right
            indices.push(desc.div * 2 + 2 + i);
            indices.push(desc.div * 2 + 2 + i + 1);
            indices.push(index_len + 2);
            // upper right
            indices.push(desc.div * 3 + 3 + i);
            indices.push(desc.div * 3 + 3 + i + 1);
            indices.push(index_len + 3);
        }

        // left
        indices.push(desc.div);
        indices.push(desc.div + 1);
        indices.push(index_len);

        indices.push(desc.div + 1);
        indices.push(index_len + 1);
        indices.push(index_len);
        // bottom
        indices.push(desc.div * 2 + 1);
        indices.push(desc.div * 2 + 2);
        indices.push(index_len + 1);

        indices.push(desc.div * 2 + 2);
        indices.push(index_len + 2);
        indices.push(index_len + 1);
        // right
        indices.push(desc.div * 3 + 2);
        indices.push(desc.div * 3 + 3);
        indices.push(index_len + 2);

        indices.push(desc.div * 3 + 3);
        indices.push(index_len + 3);
        indices.push(index_len + 2);
        // top
        indices.push(desc.div * 4 + 3);
        indices.push(0);
        indices.push(index_len + 3);

        indices.push(0);
        indices.push(index_len);
        indices.push(index_len + 3);

        (vertex, indices)
    } else {
        rectangle(
            RectangleDescriptor::new(desc.width, desc.height)
                .offset(desc.x, desc.y)
                .radius(desc.radius),
        )
    }
}

fn make_rectangle_vertex(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    div: u16,
) -> Vec<Vertex> {
    let mut vertex = Vec::with_capacity(div as usize * 4 + 4);

    // arrangement of vertices
    //
    // round A ---- round D
    //       |            |
    //       |            |
    //       |            |
    // round B ---- round C

    // Vertices

    // A

    for i in 0..=div {
        let angle = std::f32::consts::PI / 2.0 * i as f32 / div as f32;
        let x = x + radius * (1.0 - angle.sin());
        let y = -y - radius * (1.0 - angle.cos());
        vertex.push(Vertex {
            position: [x, y, 0.0],
        });
    }

    // B

    for i in 0..=div {
        let angle = std::f32::consts::PI / 2.0 * i as f32 / div as f32;
        let x = x + radius * (1.0 - angle.cos());
        let y = -y - height + radius * (1.0 - angle.sin());
        vertex.push(Vertex {
            position: [x, y, 0.0],
        });
    }

    // C

    for i in 0..=div {
        let angle = std::f32::consts::PI / 2.0 * i as f32 / div as f32;
        let x = x + width - radius * (1.0 - angle.sin());
        let y = -y - height + radius * (1.0 - angle.cos());
        vertex.push(Vertex {
            position: [x, y, 0.0],
        });
    }

    // D

    for i in 0..=div {
        let angle = std::f32::consts::PI / 2.0 * i as f32 / div as f32;
        let x = x + width - radius * (1.0 - angle.cos());
        let y = -y - radius * (1.0 - angle.sin());
        vertex.push(Vertex {
            position: [x, y, 0.0],
        });
    }

    vertex
}
