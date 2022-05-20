

use std::any::TypeId;

use crate::runtime::RUNTIME;
use super::object::JObject;

pub(crate) enum PrototypeKind{
    Object,
    Array,
    Funtion,
    String,
    Number,
    BigInt,
    Symbol,
    Promise,
}

pub(crate) fn resolve_prototype(typeid:TypeId) -> *mut JObject{
    todo!();
    RUNTIME.with(|runtime|{

    });
    if typeid == TypeId::of::<JObject>(){

    }
    return 0 as _
}