use tea_ui::{
    app::App,
    component::{Component, ComponentAccess},
    types::{
        color::Color,
        size::{Size, SizeUnit},
    },
    ui::{column::Column, super_simple_button::SuperSimpleButton, teacup::Teacup, Dom},
};

fn update(component: ComponentAccess<u32>, message: ()) {}

fn view(_: &u32) -> Box<dyn Dom<()>> {
    Box::new(
        SuperSimpleButton::new(())
            .size(Size {
                width: SizeUnit::Pixel(100.0),
                height: SizeUnit::Pixel(100.0),
            })
            .background_color(Color::Rgb8USrgb {
                r: 128,
                g: 128,
                b: 128,
            }),
    )
}

fn main() {
    let component = Component::new(None, 0, update, view);

    App::new(component).run();
}
