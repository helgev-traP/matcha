// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ops::Div;

use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{color::Color, size::Size},
    ui::Dom,
    widgets::{
        container::{
            layout::Layout,
            style::{Border, Style, Visibility},
            Container, ContainerDescriptor,
        },
        div_size::DivSize,
        grid::{Grid, GridDescriptor, GridItem},
        square::{Square, SquareDescriptor},
    },
};

#[derive(Clone, Copy)]
enum Message {
    Increase,
    Decrease,
}

fn update(component: &ComponentAccess<i32>, message: Message) {
    println!("This will not be executed");
    let mut model = component.model_mut();
    match message {
        Message::Increase => *model += 1,
        Message::Decrease => *model -= 1,
    }
}

fn local_update(
    component: &ComponentAccess<i32>,
    event_result: tea_ui::events::UiEventResult<Message>,
) -> tea_ui::events::UiEventResult<Message> {
    if let Some(event) = event_result.user_event {
        let mut model = component.model_mut();
        match event {
            Message::Increase => *model += 1,
            Message::Decrease => *model -= 1,
        }
    }
    Default::default()
}

fn view(model: &i32) -> Box<dyn Dom<Message>> {
    Grid::new(GridDescriptor {
        label: None,
        template_columns: vec![DivSize::Grow(1.0), DivSize::Grow(2.0), DivSize::Grow(3.0)],
        template_rows: vec![DivSize::Grow(1.0), DivSize::Grow(2.0), DivSize::Grow(3.0)],
        gap_columns: DivSize::Pixel(10.0),
        gap_rows: DivSize::Pixel(10.0),
        items: vec![
            GridItem {
                column: [0, 0],
                row: [0, 0],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 255, g: 0, b: 0 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [1, 1],
                row: [0, 0],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 255, g: 100, b: 100 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [2, 2],
                row: [0, 0],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 255, g: 200, b: 200 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [0, 0],
                row: [1, 1],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 0, g: 255, b: 0 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [1, 1],
                row: [1, 1],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 100, g: 255, b: 100 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [2, 2],
                row: [1, 1],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 200, g: 255, b: 200 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [0, 0],
                row: [2, 2],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 255 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [1, 1],
                row: [2, 2],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 100, g: 100, b: 255 },
                    ..Default::default()
                }),
            },
            GridItem {
                column: [2, 2],
                row: [2, 2],
                item: Square::new(SquareDescriptor {
                    size: [Size::Parent(1.0), Size::Parent(1.0)],
                    background_color: Color::Rgb8USrgb { r: 200, g: 200, b: 255 },
                    ..Default::default()
                }),
            },
        ],
    })
}

fn main() {
    let component = Component::new(None, 0, update, view).inner_update(local_update);

    App::new(component)
        .base_color(Color::Rgb8USrgb {
            r: 0,
            g: 0,
            b: 0,
        })
        .title("matcha UI")
        .run();
}
