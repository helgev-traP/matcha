use tea_ui;

#[tokio::main]
async fn main() {
    let mut app = tea_ui::app::App::new();
    let teacup = tea_ui::widgets::teacup::Teacup::new();
    app.run();
}
