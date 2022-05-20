
use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::{value::JValue, vm::VmContext};
use crate::heap;
use crate::operator;

use super::prototypes::{
    resolve_prototype,
    PrototypeKind
};
use super::*;

pub trait JObjectInner:Any + Sync + Send{
    fn call(&mut self, vmctx:&mut VmContext, this:JValue, args:&[JValue]) -> JValue{
        type_name::<Self>();
        operator::throw(JValue::Undefined)
    }

    fn get(&mut self, name:&str) -> Option<JValue>{
        return None
    }

    fn set(&mut self, name:&str, value:JValue) -> bool{
        return false
    }
}

impl dyn JObjectInner{
    pub fn downcast_ref<T>(&self) -> Option<&mut T> where T:JObjectInner{
        if self.type_id() == TypeId::of::<T>(){
            Some(unsafe{(self as *const dyn JObjectInner as *mut T).as_mut().unwrap()})
        } else{
            None
        }
    }

    pub fn borrow_mut(&self) -> &mut dyn JObjectInner{
        unsafe{(self as *const Self as *mut Self).as_mut().unwrap()}
    }

    pub fn is_array(&self) -> bool{
        self.type_id() == TypeId::of::<Array>()
    }
}


pub struct JObject{

    prototype:*mut JObject,

    values:HashMap<String, JValue>,

    freezed:bool,
    extendable:bool,

    pub(crate) inner:Option<Arc<dyn JObjectInner>>
}

impl RefUnwindSafe for JObject{}
impl UnwindSafe for JObject{}
unsafe impl Send for JObject{}
unsafe impl Sync for JObject{}

impl JObject{

    pub fn new() -> &'static mut JObject{
        let ptr = heap::malloc::<Self>();

        *ptr = JObject { 

            prototype: resolve_prototype(TypeId::of::<JObject>()), 
            values:HashMap::default(),

            freezed: false, 
            extendable: true, 
            inner: None
        };
        ptr
    }

    pub unsafe fn construct() -> JValue{
        let ptr = heap::malloc::<Self>();

        *ptr = JObject { 

            prototype: resolve_prototype(TypeId::of::<JObject>()), 
            values:HashMap::default(),

            freezed: false, 
            extendable: true, 
            inner: None
        };
        return JValue::Object(ptr)
    }

    /// determine the prototype of object
    pub fn fromInner(inner:Arc<dyn JObjectInner>) -> &'static mut JObject{
        let ptr = heap::malloc::<Self>();

        *ptr = JObject{ 
            prototype: resolve_prototype(inner.type_id()), 
            values:HashMap::default(),

            freezed: false, 
            extendable: true, 
            inner: Some(inner)
        };
        return ptr
    }

    pub fn inner_ref_uncheck(&self) -> &Arc<dyn JObjectInner>{
        self.inner.as_ref().unwrap()
    }

    pub fn member_str(&mut self, name:&str) -> JValue{
        JValue::Undefined
    }

    pub fn set_member_str<T>(&mut self, name:&str, value:T) where T:Into<JValue>{
        let value = value.into();
        if let Some(iner) = &self.inner{
            if !iner.borrow_mut().set(name, value){
                self.values.insert(name.to_string(), value);
            }
        } else{
            self.values.insert(name.to_string(), value);
        }
    }

    pub(crate) fn builtin_member<T>(&mut self, name:&str, value:T) where T:Into<JValue>{
        let value = value.into();
        if let Some(iner) = &self.inner{
            if !iner.borrow_mut().set(name, value){
                self.values.insert(name.to_string(), value);
            }
        } else{
            self.values.insert(name.to_string(), value);
        }
    }
}

impl Hash for JObject{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.prototype as usize);
        state.write_u8(self.extendable as u8);
        state.write_u8(self.freezed as u8);
        state.write_u8(self.inner.is_some() as u8);
        if let Some(i) = &self.inner{
            state.write_usize(i.as_ref() as *const _ as *mut u8 as usize);
        } else{
            state.write_usize(0);
        }
        state.write_usize(self.values.len());
        state.write_usize(self.values.capacity());
        state.write_usize(self as *const Self as usize);
    }
}