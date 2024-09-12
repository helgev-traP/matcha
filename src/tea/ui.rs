pub trait Ui {
    fn set_application_context(&mut self, context: crate::application_context::ApplicationContext);
    fn size(&self) -> crate::types::Size;
    fn resize(&mut self, size: crate::types::Size);
}

pub struct WidgetRenderObject<'a> {
    pub size: &'a crate::types::Size,
    pub offset: [f32; 2], // this will be replaced by 2D affine matrix in the future.
    // pub affine: &'a [[f32; 3]; 3],
    pub vertex_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
    pub index_count: u32,
    // bind_groups: Vec<&'a wgpu::BindGroup>,
    pub texture: &'a wgpu::Texture,
}

pub trait Widgets: Ui {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>>;
}

pub trait RenderArea: Ui + Widgets {
    fn render(&self) -> Option<&wgpu::Texture>;
}

pub enum Layout {
    Column(Vec<Box<dyn Widgets>>),
    Row(Vec<Box<dyn Widgets>>),
    Grid(Vec<Vec<Box<dyn Widgets>>>),
}

impl Ui for Layout {
    fn set_application_context(&mut self, context: crate::application_context::ApplicationContext) {
        match self {
            Layout::Column(widgets) | Layout::Row(widgets) => {
                for widget in widgets {
                    widget.set_application_context(context.clone());
                }
            }
            Layout::Grid(widgets) => {
                for row in widgets {
                    for widget in row {
                        widget.set_application_context(context.clone());
                    }
                }
            }
        }
    }

    fn size(&self) -> crate::types::Size {
        match self {
            Layout::Column(widgets) => {
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                for widget in widgets {
                    let size = widget.size();
                    width = width.max(size.width);
                    height += size.height;
                }
                crate::types::Size { width, height }
            }
            Layout::Row(widgets) => {
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                for widget in widgets {
                    let size = widget.size();
                    width += size.width;
                    height = height.max(size.height);
                }
                crate::types::Size { width, height }
            }
            Layout::Grid(widgets) => {
                let mut width: f32 = 0.0;
                let mut height: f32 = 0.0;
                for row in widgets {
                    let mut row_width: f32 = 0.0;
                    let mut row_height: f32 = 0.0;
                    for widget in row {
                        let size = widget.size();
                        row_width += size.width;
                        row_height = row_height.max(size.height);
                    }
                    width = width.max(row_width);
                    height += row_height;
                }
                crate::types::Size { width, height }
            }
        }
    }

    fn resize(&mut self, _size: crate::types::Size) {}
}

impl Widgets for Layout {
    fn render_object(&self) -> Option<Vec<WidgetRenderObject>> {
        match self {
            Layout::Column(widgets) => {
                let mut render_objects = Vec::new();
                let mut offset_y = 0.0;

                for widget in widgets {
                    let size = widget.size();
                    let render_object = widget.render_object()?;

                    for object in render_object {
                        render_objects.push(WidgetRenderObject {
                            size: object.size,
                            offset: [object.offset[0], object.offset[1] - offset_y],
                            vertex_buffer: object.vertex_buffer,
                            index_buffer: object.index_buffer,
                            index_count: object.index_count,
                            texture: object.texture,
                        });
                    }
                    offset_y -= size.height;
                }
                Some(render_objects)
            },
            Layout::Row(widgets) => {
                let mut render_objects = Vec::new();
                let mut offset_x = 0.0;

                for widget in widgets {
                    let size = widget.size();
                    let render_object = widget.render_object()?;

                    for object in render_object {
                        render_objects.push(WidgetRenderObject {
                            size: object.size,
                            offset: [object.offset[0] + offset_x, object.offset[1]],
                            vertex_buffer: object.vertex_buffer,
                            index_buffer: object.index_buffer,
                            index_count: object.index_count,
                            texture: object.texture,
                        });
                    }
                    offset_x += size.width;
                }
                Some(render_objects)
            },
            Layout::Grid(widgets) => {
                let mut render_objects = Vec::new();
                let mut offset_y = 0.0;

                for row in widgets {
                    let mut offset_x = 0.0;
                    let mut row_height: f32 = 0.0;

                    for widget in row {
                        let size = widget.size();
                        let render_object = widget.render_object()?;

                        for object in render_object {
                            render_objects.push(WidgetRenderObject {
                                size: object.size,
                                offset: [object.offset[0] + offset_x, object.offset[1] - offset_y],
                                vertex_buffer: object.vertex_buffer,
                                index_buffer: object.index_buffer,
                                index_count: object.index_count,
                                texture: object.texture,
                            });
                        }
                        offset_x += size.width;
                        row_height = row_height.max(size.height);
                    }
                    offset_y -= row_height;
                }
                Some(render_objects)
            },
        }
    }
}
