#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::{
        column::{Column, ColumnDescriptor},
        drag_field::{DragField, DragFieldDescriptor},
        row::{Row, RowDescriptor},
        square::{Square, SquareDescriptor},
        teacup::{Teacup, TeacupDescriptor},
        text::{Text, TextDescriptor},
        Dom,
    },
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn Dom<()>> {
    Box::new(DragField::new(DragFieldDescriptor {
        label: None,
        size: Size {
            width: SizeUnit::Percent(100.0),
            height: SizeUnit::Percent(100.0),
        },
        item: Box::new(Square::new(SquareDescriptor {
            size: Size {
                width: SizeUnit::Pixel(500.0),
                height: SizeUnit::Pixel(200.0),
            },
            radius: 50.0,
            background_color: Color::Rgba8USrgb {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            label: None,
            border_width: 7.0,
            border_color: Color::Rgb8USrgb { r: 0, g: 255, b: 255 },
        })),
    }))
}

fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component)
        .base_color(Color::Rgb8USrgb {
            r: 30,
            g: 30,
            b: 30,
        })
        .title("matcha UI")
        .run();
}
