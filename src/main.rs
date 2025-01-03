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
        row::{Row, RowDescriptor},
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
    Row::new(RowDescriptor {
        size: [Size::Parent(1.0), Size::Pixel(100.0)],
        justify_content: tea_ui::widgets::layout::JustifyContent::SpaceEvenly,
        align_content: tea_ui::widgets::layout::AlignContent::End,
        items: vec![
            Square::new(SquareDescriptor {
                size: [Size::Pixel(50.0), Size::Pixel(50.0)],
                background_color: Color::Rgb8USrgb { r: 255, g: 0, b: 0 },
                ..Default::default()
            }),
            Square::new(SquareDescriptor {
                size: [Size::Pixel(30.0), Size::Pixel(30.0)],
                background_color: Color::Rgb8USrgb { r: 0, g: 255, b: 0 },
                ..Default::default()
            }),
            Square::new(SquareDescriptor {
                size: [Size::Pixel(100.0), Size::Pixel(100.0)],
                background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 255 },
                ..Default::default()
            }),
        ],
        ..Default::default()
    })
}

fn main() {
    let component = Component::new(None, 0, update, view).inner_update(local_update);

    App::new(component)
        .base_color(Color::Rgb8USrgb { r: 0, g: 0, b: 0 })
        .title("matcha UI")
        .run();
}
