#![warn(missing_docs)]

pub mod bar;
pub mod buffer;
pub mod completion;
pub mod config;
pub mod config_options;
pub mod hashtable;
pub mod hdata;
pub mod hooks;
pub mod infolist;
pub mod plugin;
pub mod weechat;

pub use weechat_macro::weechat_plugin;

pub use plugin::{WeechatPlugin, WeechatResult};
pub use weechat::{ArgsWeechat, OptionChanged, Weechat};

pub use buffer::{Buffer, Nick, NickArgs};

pub use config::{Config, ConfigSection, ConfigSectionInfo};
pub use config_options::{
    BooleanOption, ColorOption, ConfigOption, IntegerOption, StringOption,
};

pub use hooks::{
    CommandDescription, CommandHook, CommandRunHook, FdHook, FdHookMode,
    SignalHook, SignalHookValue, TimerHook,
};

pub use completion::{Completion, CompletionHook, CompletionPosition};
pub use hashtable::{Hashtable, HashtableItemType};
pub use hdata::HasHData;
pub use infolist::Infolist;

use std::ffi::CString;

/// Status values for weechat callbacks
pub enum ReturnCode {
    Ok = weechat_sys::WEECHAT_RC_OK as isize,
    OkEat = weechat_sys::WEECHAT_RC_OK_EAT as isize,
    Error = weechat_sys::WEECHAT_RC_ERROR as isize,
}

pub(crate) struct LossyCString;

impl LossyCString {
    pub(crate) fn new<T: AsRef<str>>(t: T) -> CString {
        match CString::new(t.as_ref()) {
            Ok(cstr) => cstr,
            Err(_) => CString::new(t.as_ref().replace('\0', ""))
                .expect("string has no nulls"),
        }
    }
}

/// A sealed type, allowing thread-unsafe weechat types to be safely
/// passed between threads.
///
/// Sealed items can be created with the `sealed` function on some types,
/// and then unsealed with the `unseal` function and a reference to the
/// `Weechat` object.
///
/// If the sealed object has been sent to a background thread, then to obtain
/// a weechat object you must use the `on_main` or `on_main_blocking` functions
/// to run code on the main thread with a reference to the `Weechat` object.
pub struct Sealed<T>(T);

unsafe impl<T> Send for Sealed<T> {}
unsafe impl<T> Sync for Sealed<T> {}

impl<T> Sealed<T> {
    /// Unseal an object, returning the sealed object.
    ///
    /// This requires a `Weechat` object, and because it is !Send
    /// you must use the `on_main` function to safely obtain a Weechat
    /// object.
    ///
    /// The Weechat reference is not used and serves only as a token
    /// to ensure the function is called on the main thread.
    pub fn unseal(self, _: &weechat::Weechat) -> T {
        self.0
    }
}
