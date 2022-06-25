use std::panic::{catch_unwind, panic_any};

use crate::builtins::object::JObjectInnerEnum;
use crate::value::JValue;
use crate::builtins::array::Array;



pub fn throw(value:JValue) -> !{
    panic_any(value)
}

pub fn check_throw(value:JValue, ok:bool){
    if !ok{
        throw(value)
    }
}

pub fn Await(val:JValue) -> JValue{
    todo!()
}

pub fn IteratorCollect(value:JValue) -> Vec<JValue>{
    if let Some(o) = value.object(){
        match &o.inner{
            JObjectInnerEnum::Array(a) => a.values.clone(),
            _ => Vec::new()
        }
    } else{
        Vec::new()
    }
}