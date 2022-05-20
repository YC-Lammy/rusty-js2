use std::panic::{catch_unwind, panic_any};

use crate::value::JValue;
use crate::builtins::array::Array;



pub fn throw(value:JValue) -> !{
    panic_any(value)
}

pub fn IteratorCollect(value:JValue) -> Vec<JValue>{
    if let Some(o) = value.object(){
        if let Some(i) = &o.inner{

            if let Some(arr) = i.downcast_ref::<Array>(){
                return arr.values.clone()
            }
        }
    }
    Vec::new()
}