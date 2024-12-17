// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::Dom,
    widgets::{
        column::{Column, ColumnDescriptor}, drag_field::{DragField, DragFieldDescriptor}, row::{Row, RowDescriptor}, square::{Square, SquareDescriptor}, teacup::{Teacup, TeacupDescriptor}, text::{Text, TextDescriptor}
    },
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn Dom<()>> {
    Box::new(Column::new(ColumnDescriptor {
        label: None,
        vec: vec![
            Box::new(Square::new(SquareDescriptor {
                label: None,
                size: Size {
                    width: SizeUnit::Pixel(100.0),
                    height: SizeUnit::Pixel(100.0),
                },
                radius: 30.0,
                background_color: Color::Rgb8USrgb { r: 255, g: 255, b: 255 },
                border_width: 2.0,
                border_color: Color::Rgb8USrgb { r: 255, g: 0, b: 0 },
            })),
            Box::new(Text::new(TextDescriptor {
                label: None,
                size: Size {
                    width: SizeUnit::Pixel(100.0),
                    height: SizeUnit::Pixel(100.0),
                },
                font_size: 30.0,
                font_color: Color::Rgb8USrgb { r: 255, g: 255, b: 255 },
                text: "Hello!".to_string(),
                editable: false,
            })),
            Box::new(Teacup::new(TeacupDescriptor {
                label: None,
                size: Size {
                    width: SizeUnit::Pixel(100.0),
                    height: SizeUnit::Pixel(100.0),
                },
                frame_size: Size {
                    width: SizeUnit::Pixel(100.0),
                    height: SizeUnit::Pixel(100.0),
                },
                position: [0.0, 0.0],
                rotate: 0.0,
                visible: true,
            })),
        ],
    }))
}

fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component)
        .base_color(Color::Rgb8USrgb {
            r: 50,
            g: 50,
            b: 50,
        })
        .title("matcha UI")
        .run();
}
