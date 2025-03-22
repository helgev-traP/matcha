pub mod margin;
pub use margin::*;
pub mod padding;
pub use padding::*;
pub mod border;
pub use border::*;
pub mod box_sizing;
pub use box_sizing::*;

pub mod font;
pub use font::*;

pub mod visibility;
pub use visibility::*;
pub mod cursor;
pub use cursor::*;

use crate::types::{color::Color, size::Size};

#[derive(Debug, Clone)]
pub struct Style {
    // size
    pub size: [Size; 2],

    // box model
    pub margin: Margin,
    pub padding: Padding,
    pub border: Border,
    pub box_sizing: BoxSizing,

    // colors
    pub text_color: Color,
    pub background_color: Color,
    // pub background_image: Option<crate::types::Image>,

    // font
    pub font_family: String,
    pub font_size: f32,
    pub line_height_em: f32,
    pub letter_spacing: f32,
    pub font_weight: u32,
    pub font_style: FontStyle,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,

    // else
    pub opacity: u8,
    pub visibility: Visibility,
    pub cursor: Cursor,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            // size
            size: [Size::Content(1.0), Size::Content(1.0)],

            // box model
            margin: Margin::default(),
            padding: Padding::default(),
            border: Border::default(),
            box_sizing: BoxSizing::default(),

            // colors
            text_color: Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            background_color: Color::Rgba8USrgb {
                r: 255,
                g: 255,
                b: 255,
                a: 0,
            },

            // font
            font_family: "Arial".to_string(),
            font_size: 16.0,
            line_height_em: 1.5,
            letter_spacing: 0.0,
            font_weight: 400,
            font_style: FontStyle::default(),
            text_align: TextAlign::default(),
            text_decoration: TextDecoration::default(),

            // else
            opacity: 255,
            visibility: Visibility::default(),
            cursor: Cursor::default(),
        }
    }
}
