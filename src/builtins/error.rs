use std::sync::Arc;

use crate::value::JValue;

use super::object::{JObjectInner, JObject, JObjectInnerEnum};

pub trait Named {
    fn name(&self) -> &str;
}

pub struct Error{
    name:String,
    message:String
}

impl Error{
    pub fn newTypeError<S>(message:S) -> JValue where S:Into<String>{
        let obj = JObject::new();
        obj.inner = JObjectInnerEnum::Error(Error{
            name:"TypeError".into(),
            message:message.into()
        });
        JValue::Object(obj)
    }
}