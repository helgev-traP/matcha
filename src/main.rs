// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{color::Color, size::Size},
    ui::Dom,
    vertex::vertex_generator::border,
    widgets::{
        position::Position,
        row::{Row, RowDescriptor},
        square::{Square, SquareDescriptor},
        style::Border,
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
    let mut position = Position::new(None)
        .size(Size::Parent(1.0), Size::Parent(1.0))
        .background_color([0.5, 0.5, 0.5, 1.0].into())
        .border(Border::new(20.0, [255, 0, 0].into()).radius(50.0));

    // randomly place a lot of squares

    for _ in 0..0 {
        position.push(
            [
                Size::Pixel(rand::random::<f32>() * 800.0),
                Size::Pixel(rand::random::<f32>() * 600.0),
            ],
            Square::new(SquareDescriptor {
                size: [
                    Size::Pixel(rand::random::<f32>() * 300.0),
                    Size::Pixel(rand::random::<f32>() * 300.0),
                ],
                background_color: [
                    rand::random::<u8>(),
                    rand::random::<u8>(),
                    rand::random::<u8>(),
                    255,
                ]
                .into(),
                ..Default::default()
            }),
        );
    }

    position
}

fn main() {
    let component = Component::new(None, 0, update, view).inner_update(local_update);

    App::new(component)
        .base_color(Color::Rgb8USrgb { r: 0, g: 0, b: 0 })
        .title("matcha UI")
        .run();
}
