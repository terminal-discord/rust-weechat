use std::borrow::Cow;
use std::ffi::CStr;
use std::marker::PhantomData;

use crate::{Buffer, LossyCString, Weechat};
use weechat_sys::{t_gui_buffer, t_gui_nick, t_weechat_plugin};

pub struct NickSettings<'a> {
    /// Name of the new nick.
    pub(crate) name: &'a str,
    /// Color for the nick.
    pub(crate) color: &'a str,
    /// Prefix that will be shown before the name.
    pub(crate) prefix: &'a str,
    /// Color of the prefix.
    pub(crate) prefix_color: &'a str,
    /// Should the nick be visible in the nicklist.
    pub(crate) visible: bool,
}

impl<'a> NickSettings<'a> {
    pub fn new(name: &str) -> NickSettings {
        NickSettings {
            name,
            color: "",
            prefix: "",
            prefix_color: "",
            visible: true,
        }
    }

    pub fn set_color(mut self, color: &'a str) -> NickSettings<'a> {
        self.color = color;
        self
    }

    pub fn set_prefix(mut self, prefix: &'a str) -> NickSettings<'a> {
        self.prefix = prefix;
        self
    }

    pub fn set_prefix_color(
        mut self,
        prefix_color: &'a str,
    ) -> NickSettings<'a> {
        self.prefix_color = prefix_color;
        self
    }

    pub fn set_visible(mut self, visible: bool) -> NickSettings<'a> {
        self.visible = visible;
        self
    }
}

/// Nick creation arguments
pub struct NickArgs<'a> {
    /// Name of the new nick.
    pub name: &'a str,
    /// Color for the nick.
    pub color: &'a str,
    /// Prefix that will be shown before the name.
    pub prefix: &'a str,
    /// Color of the prefix.
    pub prefix_color: &'a str,
    /// Should the nick be visible in the nicklist.
    pub visible: bool,
}

impl<'a> Default for NickArgs<'a> {
    fn default() -> NickArgs<'a> {
        NickArgs {
            name: "",
            color: "",
            prefix: "",
            prefix_color: "",
            visible: true,
        }
    }
}

/// Weechat Nick type
pub struct Nick<'a> {
    pub(crate) ptr: *mut t_gui_nick,
    pub(crate) buf_ptr: *mut t_gui_buffer,
    pub(crate) weechat_ptr: *mut t_weechat_plugin,
    pub(crate) buffer: PhantomData<&'a Buffer<'a>>,
}

impl<'a> Nick<'a> {
    /// Get a Weechat object out of the nick.
    fn get_weechat(&self) -> Weechat {
        Weechat::from_ptr(self.weechat_ptr)
    }

    /// Get a string property of the nick.
    /// * `property` - The name of the property to get the value for, this can
    ///     be one of name, color, prefix or prefix_color. If a unknown
    ///     property is requested an empty string is returned.
    pub fn get_string(&self, property: &str) -> Option<Cow<str>> {
        let weechat = self.get_weechat();
        let get_string = weechat.get().nicklist_nick_get_string.unwrap();
        let c_property = LossyCString::new(property);
        unsafe {
            let ret = get_string(self.buf_ptr, self.ptr, c_property.as_ptr());

            if ret.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ret).to_string_lossy())
            }
        }
    }

    /// Get the name property of the nick.
    pub fn get_name(&self) -> Cow<str> {
        self.get_string("name").unwrap()
    }

    /// Removes the nick from it's nicklist
    pub fn remove(&self) {
        let weechat = self.get_weechat();

        let nicklist_remove_nick = weechat.get().nicklist_remove_nick.unwrap();

        unsafe {
            nicklist_remove_nick(self.buf_ptr, self.ptr);
        }
    }
}