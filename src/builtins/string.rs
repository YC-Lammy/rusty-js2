use std::ops::{Add, Deref};
use std::hash::Hash;


use crate::value::JValue;
use crate::string_allocator::STRING_ALLOCATOR;
use crate::allocator::DataMarker;

/// JString is a string allocated on a runtime local allocator.
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct JString{
    len:u32,
    ptr:*const u8,
}

impl JString{
    pub fn len(&self) -> usize{
        self.len as usize
    }

    pub fn as_str(&self) -> &str{
        self.as_ref()
    }
}

impl Add<JString> for &str{
    type Output = JString;
    fn add(self, rhs: JString) -> Self::Output {
        unsafe{
            let ptr = STRING_ALLOCATOR.lock().alloc(std::alloc::Layout::array::<u8>(self.len()+rhs.len()).unwrap());
            std::ptr::copy_nonoverlapping(self.as_ptr(), ptr, self.len());
            std::ptr::copy_nonoverlapping(rhs.ptr, ptr.add(self.len()), rhs.len());

            JString{
                ptr:ptr,
                len:self.len() as u32 + rhs.len
            }
        }
    }
}

impl AsRef<str> for JString{
    fn as_ref(&self) -> &str {

        unsafe{std::mem::transmute(std::slice::from_raw_parts::<'static,u8>(self.ptr, self.len()))}
    }
}

impl Deref for JString{
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Hash for JString{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl From<&str> for JValue{
    fn from(s: &str) -> Self {
        unsafe{
            let ptr = STRING_ALLOCATOR.lock().alloc(std::alloc::Layout::array::<u8>(s.len()).unwrap());
            std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());

            return JValue::String(JString{
                ptr:ptr,
                len:s.len() as u32
            })
        }
    }
}

impl From<String> for JValue{
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}