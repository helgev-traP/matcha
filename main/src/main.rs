// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, vec};

use matcha_widgets::widget::primitives::layout::row;
use text_render::text::{TextContext, TextRenderConfig};

fn main() {
    let mut text = TextContext::new(50, 50, 50);

    let layout = text
        .layout(
            "Hello, world!",
            TextRenderConfig {
                font: fontdb::Query {
                    families: &[fontdb::Family::SansSerif],
                    ..Default::default()
                },
                font_size: 10.0,
                line_length: 100.0,
                ..Default::default()
            },
        )
        .unwrap();

    println!("{:?}", layout.bounds);
    layout.glyphs.into_iter().for_each(|glyph| {
        println!("{:?}", glyph);
    });
}
