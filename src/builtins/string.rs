use std::ops::{Add, Deref};
use std::hash::Hash;


use crate::value::JValue;
use crate::string_allocator::STRING_ALLOCATOR;


pub struct JStringInner{
    count:u16
}

#[derive(Clone, Copy)]
pub struct JString(pub(crate) *const u8);

impl JString{
    pub fn len(&self) -> usize{
        unsafe{
            let mut p = self.0;
            let mut i = 0;
            while *p!= 0{
                p = p.add(1);
                i += 1;
            }
            i
        }
    }
}

/*
impl Drop for JString{
    fn drop(&mut self) {
        unsafe{
            
            std::alloc::dealloc(self.0 as *mut u8, std::alloc::Layout::array::<u8>(self.len()).unwrap());
        }
        
    }
}
*/

impl Add<JString> for &str{
    type Output = JString;
    fn add(self, rhs: JString) -> Self::Output {
        unsafe{
            let ptr = std::alloc::alloc_zeroed(std::alloc::Layout::array::<u8>(self.len()+rhs.len()+1).unwrap());
            std::ptr::copy_nonoverlapping(self.as_ptr(), ptr, self.len());
            std::ptr::copy_nonoverlapping(rhs.0, ptr.add(self.len()), rhs.len());
            JString(ptr)
        }
    }
}

impl AsRef<str> for JString{
    fn as_ref(&self) -> &str {

        unsafe{std::mem::transmute(std::slice::from_raw_parts::<'static,u8>(self.0, self.len()))}
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
        unsafe{state.write(std::slice::from_raw_parts(self.0, self.len()))}
    }
}

impl From<&str> for JValue{
    fn from(s: &str) -> Self {
        unsafe{
            let ptr = STRING_ALLOCATOR.lock().alloc(std::alloc::Layout::array::<u8>(s.len() + 1).unwrap());
            std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
            return JValue::String(JString(ptr))
        }
    }
}

impl From<String> for JValue{
    fn from(s: String) -> Self {
        unsafe{
            let ptr = STRING_ALLOCATOR.lock().alloc(std::alloc::Layout::array::<u8>(s.len() + 1).unwrap());
            std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
            return JValue::String(JString(ptr))
        }
    }
}