use crate::buffer::Buffer;

/// The priority of the hotlist
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum HotlistPriority {
    Low,
    Message,
    Private,
    Highlight,
}

impl HotlistPriority {
    fn to_c_rep(&self) -> &'static str {
        use HotlistPriority::*;
        match self {
            Low => "0",
            Message => "1",
            Private => "2",
            Highlight => "3",
        }
    }
}

impl Buffer<'_> {
    /// Remove buffer from the hotlist.
    pub fn clear_hotlist(&self) {
        self.set("hotlist", "-1");
    }

    /// Enable hotlist
    pub fn enable_hotlist(&self) {
        self.set("hotlist", "+");
    }

    /// Disable hotlist
    pub fn disable_hotlist(&self) {
        self.set("hotlist", "-");
    }

    /// Add buffer to the hotlist.
    pub fn set_hotlist(&self, priority: HotlistPriority) {
        self.set("hotlist", priority.to_c_rep());
    }
}
