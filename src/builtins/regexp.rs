use std::sync::Arc;

use crate::value::JValue;

use super::{JObjectInner, JString, JObject};




pub struct RegExp{

}

impl RegExp{
    pub fn from_str(exp:&str, flags:&str) -> JValue{
        todo!()
    }
}

impl JObjectInner for RegExp{

}