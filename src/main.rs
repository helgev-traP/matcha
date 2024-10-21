use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::{
        column::{Column, ColumnDescriptor}, drag_field::{DragField, DragFieldDescriptor}, row::{Row, RowDescriptor}, square::{Square, SquareDescriptor}, teacup::{Teacup, TeacupDescriptor}, Dom
    },
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn Dom<()>> {
    Box::new(
        DragField::new(
            DragFieldDescriptor {
                label: None,
                size: Size {
                    width: SizeUnit::Percent(100.0),
                    height: SizeUnit::Percent(100.0),
                },
                item: Box::new(
                    Square::new(
                        SquareDescriptor {
                            label: None,
                            size: Size {
                                width: SizeUnit::Pixel(100.0),
                                height: SizeUnit::Pixel(100.0),
                            },
                            background_color: Color::Rgb8USrgb {
                                r: 255,
                                g: 0,
                                b: 0,
                            },
                        }
                    )
                ),
            }
        )
    )
}

fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component)
        .base_color(Color::Rgb8USrgb {
            r: 0,
            g: 0,
            b: 0,
        })
        .title("traP Conference")
        .run();
}
