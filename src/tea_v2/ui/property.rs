pub mod display;
pub use display::*;
pub mod position;
pub use position::*;
pub mod overflow;
pub use overflow::*;

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

#[derive(Debug, Clone)]
pub struct Property {
    // layouts
    pub display: Display,
    pub grid_item: GritItem,
    pub position: Position,
    pub overflow: Overflow,

    // size
    pub size: crate::types::Size,

    // box model
    pub margin: Margin,
    pub padding: Padding,
    pub border: Border,
    pub box_sizing: BoxSizing,

    // colors
    pub text_color: crate::types::Color,
    pub background_color: crate::types::Color,
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
    // pub transition: Transition,
}

impl Default for Property {
    fn default() -> Self {
        Self {
            // layouts
            display: Display::default(),
            grid_item: GritItem {
                row: [0, 0],
                column: [0, 0],
            },
            position: Position::default(),
            overflow: Overflow::default(),

            // size
            size: crate::types::Size {
                width: crate::types::SizeUnit::Content(1.0),
                height: crate::types::SizeUnit::Content(1.0),
            },

            // box model
            margin: Margin::default(),
            padding: Padding::default(),
            border: Border::default(),
            box_sizing: BoxSizing::default(),

            // colors
            text_color: crate::types::Color::Rgb8USrgb { r: 0, g: 0, b: 0 },
            background_color: crate::types::Color::Rgb8USrgb { r: 255, g: 255, b: 255 },

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

