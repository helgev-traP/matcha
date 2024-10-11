use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::size::{Size, SizeUnit},
    ui::{column::Column, teacup::Teacup, DomNode},
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn DomNode<()>> {
    let mut column = Box::new(Column::new());
    for i in 0..10 {
        column.push(Box::new(Teacup::new().size(Size {
            width: SizeUnit::Pixel(50.0 * i as f32),
            height: SizeUnit::Pixel(50.0 * i as f32),
        })));
    }
    column
}

#[tokio::main]
async fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component).run();
}
