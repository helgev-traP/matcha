#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}