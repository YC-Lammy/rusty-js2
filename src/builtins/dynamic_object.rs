use std::any::{
    TypeId,
    Any
};


use crate::prelude::JValue;
use crate::prelude::OwnedValue;


pub trait DynamicObject:Any{
    

    fn set(&mut self, key:&str, value:JValue<'_>) -> bool;
    fn get(&mut self, key:&str) -> OwnedValue;
    fn ownedKeys(&self) -> Vec<String>;
}

impl dyn DynamicObject{
    fn downcast_ref<T>(&self) -> Option<&mut T> where T:DynamicObject{
        if self.type_id() == TypeId::of::<T>(){
            Some(unsafe{(self as *const Self as *mut T).as_mut().unwrap()})
        } else{
            None
        }
    }
}