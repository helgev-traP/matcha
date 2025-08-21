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
        matcha_widgets::Plain::new(None)
            .style(Box::new(
                matcha_widgets::style::image::Image::new(include_bytes!(
                    "assets/videoframe_21710.png"
                ))
                .size([
                    matcha_widgets::types::size::Size::child_w(1.0),
                    matcha_widgets::types::size::Size::child_h(1.0),
                ]),
            ))
            .boundary_size([200.0, 100.0]),
    ) as Box<dyn Dom<MyMessage>>
}

#[allow(clippy::unwrap_used)]
fn main() {
    let component = Component::<MyModel, (), MyEvent, MyMessage>::new(
        Some("Counter"),
        MyModel { count: 0 },
        view,
    )
    .react_fn(|msg, model_accessor| -> Option<MyEvent> {
        update(&msg, model_accessor);
        None
    });
    App::new(component).run().unwrap();
}
