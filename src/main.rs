use tea_ui::{self, cosmic, types::Size, ui::Layout, widgets::text::Text};

#[tokio::main]
async fn main() {
    let mut app = tea_ui::app::App::new()
        .widgets(vec![Box::new(Layout::Row(vec![
            Box::new(
                Text::new(
                    "Hello, World!".to_string(),
                    Size {
                        width: 1000.0,
                        height: 100.0,
                    },
                )
                .font_size(100.0)
                .line_height(100.0),
            ),
            Box::new(
                Text::new(
                    "Hello, World!".to_string(),
                    Size {
                        width: 1000.0,
                        height: 100.0,
                    },
                )
                .font_size(100.0)
                .line_height(100.0),
            ),
        ]))])
        .base_color([0, 0, 0, 255]);

    app.run();
}
