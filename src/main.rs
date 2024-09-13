use cgmath::num_traits::float;
use tea_ui::{self, cosmic, types::Size, ui::Layout, widgets::text::Text};

#[tokio::main]
async fn main() {
    let mut app = tea_ui::app::App::new()
        .widgets(vec![Box::new(
            Text::new(
                "Hello, World!".to_string(),
                Size {
                    width: 1000.0,
                    height: 150.0,
                },
            )
            .font_size(100.0)
            .line_height(100.0),
        )])
        .base_color([10, 10, 10, 255]);

    let top_panel = app.add_top_panel(100.0);

    top_panel.set_base_color([230, 230, 230, 255]);
    top_panel.add_widget(Box::new(
        Text::new(
            "top panel".to_string(),
            Size {
                width: 1000.0,
                height: 150.0,
            },
        )
        .font_size(100.0)
        .line_height(100.0)
        .font_color([0, 0, 0, 255]),
    ));

    let left_panel = app.add_left_panel(430.0);

    left_panel.set_base_color([50, 230, 230, 255]);
    left_panel.add_widget(Box::new(
        Text::new(
            "left panel".to_string(),
            Size {
                width: 1000.0,
                height: 150.0,
            },
        )
        .font_size(100.0)
        .line_height(100.0)
        .font_color([255, 0, 0, 255]),
    ));

    let bottom_panel = app.add_bottom_panel(100.0);
    bottom_panel.set_base_color([100, 100, 100, 255]);
    bottom_panel.add_widget(Box::new(
        Text::new(
            "bottom".to_string(),
            Size {
                width: 1000.0,
                height: 150.0,
            },
        )
        .font_size(100.0)
        .line_height(100.0)
        .font_color([255, 255, 255, 255]),
    ));

    let mut floating_panel = app.add_floating_panel(800.0, -400.0, 0.0, Size {
        width: 800.0,
        height: 400.0,
    });
    floating_panel.set_base_color([255, 255, 255, 255]);
    floating_panel.add_widget(Box::new(
        Text::new(
            "floating panel".to_string(),
            Size {
                width: 1000.0,
                height: 150.0,
            },
        )
        .font_size(100.0)
        .line_height(100.0)
        .font_color([0, 0, 0, 255]),
    ));

    app.run("tea-ui");
}
