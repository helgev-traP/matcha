use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::size::{Size, SizeUnit},
    ui::{teacup::Teacup, DomNode},
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn DomNode<()>> {
    Box::new(Teacup::new().size(Size {
        width: SizeUnit::Pixel(500.0),
        height: SizeUnit::Pixel(500.0),
    }))
}

#[tokio::main]
async fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component).run();
}
