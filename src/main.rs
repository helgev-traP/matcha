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
        teacup::{Teacup, TeacupDescriptor},
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
        item: Box::new(Teacup::new(TeacupDescriptor {
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
            ..Default::default()
        })),
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
