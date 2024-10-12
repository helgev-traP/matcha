use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::{
        column::{Column, ColumnDescriptor},
        row::{Row, RowDescriptor},
        super_simple_button::{SuperSimpleButton, SuperSimpleButtonDescriptor},
        teacup::{Teacup, TeacupDescriptor},
        Dom,
    },
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn Dom<()>> {
    let mut column = Column::new(ColumnDescriptor {
        label: None,
        vec: vec![],
    });

    for _ in 0..10 {
        let mut row = Row::new(RowDescriptor {
            label: None,
            vec: vec![],
        });

        for _ in 0..10 {
            // random size
            let px = rand::random::<f32>() * 50.0 + 50.0;

            row.push(Box::new(Teacup::new(TeacupDescriptor {
                size: Size {
                    width: SizeUnit::Pixel(px),
                    height: SizeUnit::Pixel(px),
                },
                frame_size: Size {
                    width: SizeUnit::Pixel(100.0),
                    height: SizeUnit::Pixel(100.0),
                },
                // visible: rand::random::<f32>() > 0.5,
                ..Default::default()
            })));
        }

        column.push(Box::new(row));
    }

    Box::new(column)
}

fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component).base_color(Color::Rgb8USrgb { r: 128, g: 128, b: 128 }).run();
}
