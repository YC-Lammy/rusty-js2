
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

    pub(crate) inner:JObjectInnerEnum
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
            inner: JObjectInnerEnum::None
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
            inner: JObjectInnerEnum::None
        };
        return JValue::Object(ptr)
    }

    pub fn member_str(&mut self, name:&str) -> JValue{
        JValue::Undefined
    }

    pub fn set_member_str<T>(&mut self, name:&str, value:T) where T:Into<JValue>{

        let value = value.into();

        if !self.inner.set(name, value){
            self.values.insert(name.to_string(), value);
        }
    }

    pub(crate) fn builtin_member<T>(&mut self, name:&str, value:T) where T:Into<JValue>{

        let value = value.into();

        if !self.inner.set(name, value){
            self.values.insert(name.to_string(), value);
        }
    }

    pub(crate) fn keep_alive(&self, alive:bool){

    }
}

impl Hash for JObject{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.prototype as usize);
        state.write_u8(self.extendable as u8);
        state.write_u8(self.freezed as u8);
        state.write_u8(self.inner.varient() as u8);
        state.write_usize(self.values.len());
        state.write_usize(self.values.capacity());
        state.write_usize(self as *const Self as usize);
    }
}

pub(crate) enum JObjectInnerEnum{
    None,

    Array(Array),
    Function(Function),
    Error(Error),
    Date(),
    RegExp(),

    Map(),
    Set(),
    WeakMap(),
    WeakSet(),

    ArrayBuffer(),
    SharedArrayBuffer(),
    DataView(),

    Promise(Promise),
    Generator(),

    Proxy(),

    TypedArray(),
    
    Boolean(bool),
    Number(f64),
    BigInt(i64),
    Symbol(Symbol),
    String(JString),

    Custom(Arc<dyn JObjectInner>)
}

impl JObjectInnerEnum{

    pub fn varient(&self) -> u8{
        match self{
            Self::None => 0,
            Self::Array(_) => 1,
            Self::ArrayBuffer() => 2,
            Self::BigInt(_) => 3,
            Self::Boolean(_) => 4,
            Self::Custom(_) => 5,
            Self::DataView() => 6,
            Self::Date() => 7,
            Self::Error(_) => 8,
            Self::Function(_) => 9,
            Self::Generator() => 10,
            Self::Map() => 11,
            Self::Number(_) => 12,
            Self::Promise(_) => 13,
            Self::Proxy() => 14,
            Self::RegExp() => 15,
            Self::Set() => 16,
            Self::SharedArrayBuffer() => 17,
            Self::String(_) => 18,
            Self::Symbol(_) => 19,
            Self::WeakMap() => 20,
            Self::WeakSet() => 21,
            Self::TypedArray() => 22
        }
    }

    pub fn is_array(&self) -> bool{
        match self{
            Self::Array(_) => true,
            _ => false
        }
    }

    pub fn is_function(&self) -> bool{
        match self{
            Self::Function(_) => true,
            _ => false
        }
    }

    pub fn array(&self) -> Option<&Array>{
        match self{
            Self::Array(a) => Some(a),
            _ => None
        }
    }

    pub fn function(&self) -> Option<&Function>{
        match self{
            Self::Function(f) => Some(f),
            _ => None
        }
    }

    pub fn set(&self, key:&str, value:JValue) -> bool{
        match self{
            Self::Array(a) => a.set(key, value),
            Self::Proxy() => todo!(),
            Self::TypedArray() => todo!(),
            _ => false
        }
    }

    pub fn call(&self, ctx:&'static VmContext, this:JValue, args:&[JValue]) -> JValue{
        todo!()
    }
}