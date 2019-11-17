//! A safe and high level API to access HData tables

use crate::{Buffer, LossyCString, Weechat};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::{c_void, CStr};
use weechat_sys::{t_hdata, t_weechat_plugin};

/// The HData object represents a table of variables associated with an object.
///
/// An HData object can be created from any Weechat type that implements [`HasHData`] using the
/// [`get_hdata`](HasHData::get_hdata) function and the name of the hdata table you want to access.
pub struct HData {
    weechat_ptr: *mut t_weechat_plugin,
    object: *mut c_void,
    ptr: *mut t_hdata,
}

impl HData {
    /// Retrieve the value of a variable in a hdata.
    pub fn get_var<T: HDataType>(&self, name: &str) -> Option<T> {
        let weechat = Weechat::from_ptr(self.weechat_ptr);

        HDataType::hdata_value(self, name)
    }

    /// Update the value of a variable in a hdata.
    pub fn update_var<T: HDataType>(&self, name: &str, value: T) -> usize {
        let weechat = Weechat::from_ptr(self.weechat_ptr);

        HDataType::hdata_set_value(self, name, value)
    }

    /// Retrieve a variable as a string.
    ///
    /// If the data is not compatible bad things will happen.
    pub unsafe fn get_string_unchecked(
        &self,
        name: &str,
    ) -> Option<Cow<'_, str>> {
        let weechat = Weechat::from_ptr(self.weechat_ptr);
        let hdata_string = weechat.get().hdata_string.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            let ret = hdata_string(self.ptr, self.object, name.as_ptr());
            if ret.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ret).to_string_lossy())
            }
        }
    }

    /// Retrieve a variable as a integer.
    ///
    /// If the data is not compatible bad things will happen.
    pub unsafe fn get_i32_unchecked(&self, name: &str) -> i32 {
        let weechat = Weechat::from_ptr(self.weechat_ptr);
        let hdata_integer = weechat.get().hdata_integer.unwrap();

        let name = LossyCString::new(name);

        unsafe { hdata_integer(self.ptr, self.object, name.as_ptr()) }
    }

    /// Retrieve a variable as a long.
    ///
    /// If the data is not compatible bad things will happen.
    pub unsafe fn get_i64_unchecked(&self, name: &str) -> i64 {
        let weechat = Weechat::from_ptr(self.weechat_ptr);
        let hdata_long = weechat.get().hdata_long.unwrap();

        let name = LossyCString::new(name);

        unsafe { hdata_long(self.ptr, self.object, name.as_ptr()) }
    }
}

/// A trait for types that have hdata.
pub trait HasHData {
    /// Retrieve a hdata table tied to this object.
    fn get_hdata(&self, name: &str) -> Option<HData>;
}

impl HasHData for Buffer {
    fn get_hdata(&self, name: &str) -> Option<HData> {
        let hdata_get =
            Weechat::from_ptr(self.weechat).get().hdata_get.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            let hdata = hdata_get(self.weechat, name.as_ptr());
            if hdata.is_null() {
                None
            } else {
                Some(HData {
                    weechat_ptr: self.weechat,
                    object: self.ptr as *mut _,
                    ptr: hdata,
                })
            }
        }
    }
}

/// A trait for types of hdata values.
pub trait HDataType: Sized {
    /// Retrieve the value of a hdata variable by name.
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self>;

    /// Set the value of a hdata variable by name.
    // TODO: Figure out ownership issues
    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize;
}

impl HDataType for Cow<'_, str> {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_string = weechat.get().hdata_string.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_STRING as i32
            {
                return None;
            }

            let ret = hdata_string(hdata.ptr, hdata.object, name.as_ptr());
            if ret.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ret).to_string_lossy())
            }
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::String,
            )
            .unwrap();

        hashtable.set(name, &value);

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

impl HDataType for String {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        HDataType::hdata_value(hdata, name).map(Cow::into_owned)
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        HDataType::hdata_set_value(hdata, name, Cow::from(value))
    }
}

impl HDataType for char {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_char = weechat.get().hdata_char.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_CHAR as i32
            {
                return None;
            }

            let c_char = hdata_char(hdata.ptr, hdata.object, name.as_ptr());
            c_char.try_into().map(|ch: u8| ch as char).ok()
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::String,
            )
            .unwrap();

        hashtable.set(name, &value.to_string());

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

impl HDataType for i64 {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_long = weechat.get().hdata_long.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_LONG as i32
            {
                return None;
            }

            Some(hdata_long(hdata.ptr, hdata.object, name.as_ptr()))
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::Integer,
            )
            .unwrap();

        hashtable.set(name, &value.to_string());

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

impl HDataType for i32 {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_integer = weechat.get().hdata_integer.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_INTEGER as i32
            {
                return None;
            }

            Some(hdata_integer(hdata.ptr, hdata.object, name.as_ptr()))
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::Integer,
            )
            .unwrap();

        hashtable.set(name, &value.to_string());

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

impl HDataType for DateTime<Utc> {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_time = weechat.get().hdata_time.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_TIME as i32
            {
                return None;
            }

            let unix_time = hdata_time(hdata.ptr, hdata.object, name.as_ptr());
            let naive = NaiveDateTime::from_timestamp(unix_time, 0);

            Some(DateTime::from_utc(naive, Utc))
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::Integer,
            )
            .unwrap();

        hashtable.set(name, &value.timestamp().to_string());

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

/// An opaque wrapper for a pointer stored in hdata
pub struct HDataPointer {
    ptr: *mut c_void,
    weechat: *mut t_weechat_plugin,
}

impl HDataPointer {
    /// Moves a pointer to a new location in a list
    pub fn advance(&self, hdata: &HData, count: i32) -> Option<HDataPointer> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_move = weechat.get().hdata_move.unwrap();

        unsafe {
            let new_ptr = hdata_move(hdata.ptr, self.ptr, count);

            if new_ptr.is_null() {
                None
            } else {
                Some(HDataPointer {
                    ptr: new_ptr,
                    weechat: self.weechat,
                })
            }
        }
    }
}

impl HDataType for HDataPointer {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_pointer = weechat.get().hdata_pointer.unwrap();
        let hdata_get_var_type = weechat.get().hdata_get_var_type.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            if hdata_get_var_type(hdata.ptr, name.as_ptr())
                != weechat_sys::WEECHAT_HDATA_POINTER as i32
            {
                return None;
            }

            Some(HDataPointer {
                ptr: hdata_pointer(hdata.ptr, hdata.object, name.as_ptr()),
                weechat: hdata.weechat_ptr,
            })
        }
    }

    fn hdata_set_value(hdata: &HData, name: &str, value: Self) -> usize {
        let weechat = Weechat::from_ptr(hdata.weechat_ptr);
        let hdata_update = weechat.get().hdata_update.unwrap();

        let hashtable = weechat
            .new_hashtable(
                1,
                crate::HashtableItemType::String,
                crate::HashtableItemType::Integer,
            )
            .unwrap();

        hashtable.set(name, &(value.ptr as usize).to_string());

        unsafe { hdata_update(hdata.ptr, hdata.object, hashtable.ptr) as usize }
    }
}

impl HasHData for HDataPointer {
    fn get_hdata(&self, name: &str) -> Option<HData> {
        let hdata_get =
            Weechat::from_ptr(self.weechat).get().hdata_get.unwrap();

        let name = LossyCString::new(name);

        unsafe {
            let hdata = hdata_get(self.weechat, name.as_ptr());
            if hdata.is_null() {
                None
            } else {
                Some(HData {
                    weechat_ptr: self.weechat,
                    object: self.ptr as *mut _,
                    ptr: hdata,
                })
            }
        }
    }
}
