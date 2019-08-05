use crate::{Buffer, LossyCString, Weechat};
use std::borrow::Cow;
use std::ffi::{c_void, CStr};

pub trait HasHData {
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

pub trait HDataType: Sized {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self>;
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
}

impl HDataType for String {
    fn hdata_value(hdata: &HData, name: &str) -> Option<Self> {
        HDataType::hdata_value(hdata, name).map(Cow::into_owned)
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
}

pub struct HData {
    weechat_ptr: *mut weechat_sys::t_weechat_plugin,
    object: *mut c_void,
    ptr: *mut weechat_sys::t_hdata,
}

impl HData {
    pub fn get_var<T: HDataType>(&self, name: &str) -> Option<T> {
        let weechat = Weechat::from_ptr(self.weechat_ptr);

        HDataType::hdata_value(self, name)
    }
}
