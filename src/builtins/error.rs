use std::sync::Arc;

use crate::value::JValue;

use super::object::{JObjectInner, JObject};

pub trait Named {
    fn name(&self) -> &str;
}

impl<E> JObjectInner for E where E:std::error::Error + Named + Sync + Send + 'static{
    fn get(&mut self, name:&str) -> Option<crate::value::JValue> {
        
        if name == "message" {
            return Some(self.to_string().into())
        }
        if name == "name" {
            return Some(self.name().into())
        }
        None
    }
}

pub struct Error{
    name:String,
    message:String
}

impl Error{
    pub fn newTypeError<S>(message:S) -> JValue where S:Into<String>{
        let inner = Arc::new(Error{
            name:"TypeError".into(),
            message:message.into()
        });
        JObject::fromInner(inner).into()
    }
}

impl JObjectInner for Error{

}