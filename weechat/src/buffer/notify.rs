use crate::buffer::Buffer;

/// The notify level
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum NotifyLevel {
    Never,
    Highlights,
    HighlightsAndMessages,
    AllMessages,
}

impl NotifyLevel {
    fn to_c_rep(&self) -> &'static str {
        use NotifyLevel::*;
        match self {
            Never => "0",
            Highlights => "1",
            HighlightsAndMessages => "2",
            AllMessages => "3",
        }
    }
}

impl Buffer<'_> {
    /// Set notify level
    pub fn set_notify(&self, level: NotifyLevel) {
        self.set("notify", level.to_c_rep());
    }
}
