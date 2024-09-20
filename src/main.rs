use tea_ui::widgets::teacup;

#[tokio::main]
async fn main() {
    let mut app = tea_ui::app::App::new()
        .base_color(tea_ui::types::Color::Rgba8USrgb {
            r: 10,
            g: 10,
            b: 10,
            a: 255,
        })
        .ui(vec![
            Box::new(teacup::Teacup::new()
                .unwrap()
                .position([100.0, 100.0])
                .rotate(0.0)),
        ]);
    app.run();
}
