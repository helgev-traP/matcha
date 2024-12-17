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
        row::{Row, RowDescriptor},
        square::{Square, SquareDescriptor},
        teacup::{Teacup, TeacupDescriptor},
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
    let mut model = component.model_mut();
    match event_result.user_event {
        Some(Message::Increase) => *model += 1,
        Some(Message::Decrease) => *model -= 1,
        None => {}
    }
    Default::default()
}

fn view(model: &i32) -> Box<dyn Dom<Message>> {
    Row::new(RowDescriptor {
        label: None,
        vec: vec![
            Button::new(ButtonDescriptor {
                label: Some("Increase".to_string()),
                size: Size {
                    width: SizeUnit::Pixel(250.0),
                    height: SizeUnit::Pixel(100.0),
                },
                background_color: Color::Rgb8USrgb { r: 255, g: 0, b: 0 },
                border_width: 5.0,
                border_color: Color::Rgb8USrgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                hover_border_width: Some(15.0),
                onclick: Some(Message::Increase),
                content: Some(Text::new(TextDescriptor {
                    size: Size {
                        width: SizeUnit::Percent(100.0),
                        height: SizeUnit::Percent(100.0),
                    },
                    font_size: 50.0,
                    font_color: Color::Rgb8USrgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                    text: "Increase".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            Text::new(TextDescriptor {
                size: Size {
                    width: SizeUnit::Pixel(200.0),
                    height: SizeUnit::Pixel(100.0),
                },
                font_size: 100.0,
                font_color: Color::Rgb8USrgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                text: model.to_string(),
                ..Default::default()
            }),
            Button::new(ButtonDescriptor {
                label: Some("Decrease".to_string()),
                size: Size {
                    width: SizeUnit::Pixel(250.0),
                    height: SizeUnit::Pixel(100.0),
                },
                background_color: Color::Rgb8USrgb { r: 0, g: 0, b: 255 },
                border_width: 5.0,
                border_color: Color::Rgb8USrgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                hover_border_width: Some(15.0),
                onclick: Some(Message::Decrease),
                content: Some(Text::new(TextDescriptor {
                    size: Size {
                        width: SizeUnit::Percent(100.0),
                        height: SizeUnit::Percent(100.0),
                    },
                    font_size: 50.0,
                    font_color: Color::Rgb8USrgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                    text: "Decrease".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        ],
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
