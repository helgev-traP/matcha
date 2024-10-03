use tea_ui::{app::{self, App}, component::{self, Component, ComponentAccess}, ui::{teacup::Teacup, DomNode}};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn DomNode<()>> {
    Box::new(Teacup::new())
}

#[tokio::main]
async fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component).run();
}
