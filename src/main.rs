use tea_ui;

#[tokio::main]
async fn main() {
    let mut app = tea_ui::app::App::new();
    let teacup = tea_ui::widgets::teacup::Teacup::new();
    app.set_ui_tree(Box::new(teacup));
    app.set_background_color([0.01, 0.01, 0.01, 1.0]);
    app.run();
}
