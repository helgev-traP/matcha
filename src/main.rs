#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::Dom,
    widgets::{
        column::{Column, ColumnDescriptor},
        drag_field::{DragField, DragFieldDescriptor},
        row::{Row, RowDescriptor},
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
        item: Box::new(Column::new(ColumnDescriptor {
            label: None,
            vec: (0..30)
                .map(|_| {
                    Box::new(Row::new(RowDescriptor {
                        label: None,
                        vec: (0..30)
                            .map(|_| {
                                Box::new(Square::new(SquareDescriptor {
                                    label: None,
                                    size: Size {
                                        width: SizeUnit::Pixel(20.0),
                                        height: SizeUnit::Pixel(20.0),
                                    },
                                    radius: 10.0,
                                    background_color: Color::Rgb8USrgb {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                    },
                                    border_width: 2.0,
                                    border_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
                                })) as Box<dyn Dom<()>>
                            })
                            .collect(),
                    })) as Box<dyn Dom<()>>
                })
                .collect(),
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