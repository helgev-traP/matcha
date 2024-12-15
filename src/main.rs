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
        drag_field::{DragField, DragFieldDescriptor},
        square::{Square, SquareDescriptor},
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
        item: Box::new(Square::new(SquareDescriptor::default())),
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
