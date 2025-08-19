use cosmic_text::Attrs;
use matcha_core::App;
use matcha_core::component::{Component, ComponentDom, ModelAccessor};
use matcha_core::ui::Dom;
use matcha_widgets::{
    Column, Row,
    widget::{button::Button, image::Image, text::Text},
};
use std::io::Cursor;

// Model
#[derive(Clone)]
struct MyModel {
    image: image::DynamicImage,
    count: i32,
}

// Messages
#[derive(Clone)]
enum MyMessage {
    Increment,
    Decrement,
}

// Events
#[derive(Debug)]
enum MyEvent {}

fn update(message: &MyMessage, model_accessor: ModelAccessor<MyModel>) {
    let msg = message.clone();
    tokio::spawn(async move {
        model_accessor
            .update(move |mut model| match msg {
                MyMessage::Increment => model.count += 1,
                MyMessage::Decrement => model.count -= 1,
            })
            .await;
    });
}

fn view(model: &MyModel) -> Box<dyn Dom<MyMessage>> {
    Box::new(
        Column::new(None)
            .push(Image::new(model.image.clone()))
            .push(Box::new(
                Row::new(None)
                    .push(Box::new(
                        Button::new(Text::new("-")).on_click(|| MyMessage::Decrement),
                    ))
                    .push(Text::new(&model.count.to_string()))
                    .push(Box::new(
                        Button::new(Text::new("+")).on_click(|| MyMessage::Increment),
                    )),
            ))
            .push(Box::new(Text::new("Hello, Matcha!").attrs(
                Attrs::new().color(cosmic_text::Color::rgba(0, 0, 255, 255)),
            ))),
    )
}

#[allow(clippy::unwrap_used)]
fn main() {
    let img_bytes = include_bytes!("assets/videoframe_21710.png");
    let image = image::ImageReader::new(Cursor::new(img_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let component = Component::<MyModel, (), MyEvent, MyMessage>::new(
        Some("Counter"),
        MyModel { count: 0, image },
        view,
    )
    .react_fn(|msg, model_accessor| -> Option<MyEvent> {
        update(&msg, model_accessor);
        None
    });
    App::new(component).run().unwrap();
}
