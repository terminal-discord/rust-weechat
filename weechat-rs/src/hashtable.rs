//! Hashtables allow storing key value pairs.

use crate::{LossyCString, Weechat};
use std::ffi::CString;
use weechat_sys::{t_hashtable, t_weechat_plugin};

pub struct Hashtable {
    weechat_ptr: *mut t_weechat_plugin,
    pub(crate) ptr: *mut t_hashtable,
}

pub enum HashtableItemType {
    Integer,
    String,
    Pointer,
    Buffer,
    Time,
}

impl ToString for HashtableItemType {
    fn to_string(&self) -> String {
        use HashtableItemType::*;
        match self {
            Integer => "integer",
            String => "string",
            Pointer => "pointer",
            Buffer => "buffer",
            Time => "time",
        }
        .into()
    }
}

impl Weechat {
    /// Create a new hashtable with the given key and value types.
    pub fn new_hashtable(
        &self,
        size: u16,
        key_type: HashtableItemType,
        value_type: HashtableItemType,
    ) -> Option<Hashtable> {
        let hashtable_new =
            Weechat::from_ptr(self.ptr).get().hashtable_new.unwrap();

        let key_type = CString::new(key_type.to_string()).unwrap();
        let value_type = CString::new(value_type.to_string()).unwrap();

        let hashtable = unsafe {
            hashtable_new(
                size as i32,
                key_type.as_ptr(),
                value_type.as_ptr(),
                None,
                None,
            )
        };

        if hashtable.is_null() {
            None
        } else {
            Some(Hashtable {
                weechat_ptr: self.ptr,
                ptr: hashtable,
            })
        }
    }
}

impl Hashtable {
    /// Add or update an item in the hashtable.
    pub fn set(&self, key: &str, value: &str) {
        let weechat_hashtable_set = Weechat::from_ptr(self.weechat_ptr)
            .get()
            .hashtable_set
            .unwrap();

        let key = LossyCString::new(key);
        let value = LossyCString::new(value);

        unsafe {
            weechat_hashtable_set(
                self.ptr,
                key.as_ptr() as *const _,
                value.as_ptr() as *const _,
            );
        }
    }
}
