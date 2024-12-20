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
        button::{Button, ButtonDescriptor},
        column::{Column, ColumnDescriptor},
        drag_field::{DragField, DragFieldDescriptor},
        square::{Square, SquareDescriptor},
        text::{Text, TextDescriptor},
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
    Square::new(SquareDescriptor {
        size: Size {
            width:  SizeUnit::Percent(100.0),
            height: SizeUnit::Percent(100.0),
        },
        radius: 200.0,
        background_color: Color::Rgb8USrgb {
            r: 0,
            g: 255,
            b: 255,
        },
        border_width: 50.0,
        border_color: Color::Rgb8USrgb {
            r: 255,
            g: 100,
            b: 0,
        },
        ..Default::default()
    })
}

fn main() {
    let component = Component::new(None, 0, update, view).inner_update(local_update);

    App::new(component)
        .base_color(Color::Rgb8USrgb {
            r: 50,
            g: 50,
            b: 50,
        })
        .title("matcha UI")
        .run();
}
