use crate::types::size::Size;

struct Button<R> {
    size: Size,
    background_color: [u8; 4],
    text: String,
    on_click: R,
}