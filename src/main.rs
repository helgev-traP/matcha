use tea_ui;

#[tokio::main]
async fn main() {
    let mut app = tea_ui::App::new();
    let teacup = tea_ui::widgets::Teacup::new();
    app.run();
}
