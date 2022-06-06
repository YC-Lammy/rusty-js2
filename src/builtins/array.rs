


use std::sync::Arc;

use crate::utils::ToMutable;
use crate::{value::JValue, vm::VmContext};
use crate::operator;


use super::object::{
    JObject,
    JObjectInner, JObjectInnerEnum
};
use super::function::Function;

pub struct Array{
    pub(crate) values:Vec<JValue>
}

impl Array{
    pub fn new(object:&'static mut JObject, values:&[JValue]) -> JValue{
        object.inner = JObjectInnerEnum::Array(Array{
            values:values.to_vec()
        });

        JValue::Object(object)
    }

    pub unsafe fn new_raw(object:&'static mut JObject, argv:*mut JValue, argc:i64, spread:bool) -> JValue{
        let mut args = std::mem::transmute::<_, &[JValue]>((argv, argc as usize));
        if spread{
            let mut v = args.to_vec();
            v.extend(operator::IteratorCollect(args[args.len() -1]));
            return Self::new(object, &v)
        };

        Self::new(object, args)
    }
    
    pub fn set(&self, key:&str, value:JValue) -> bool{
        if let Ok(mut v) = key.parse::<i64>(){
            if v < 0{
                v = v + self.values.len() as i64;
            }
            if v < 0{
                return false
            }

            if self.values.len() < v as usize{
                self.to_mut().values.resize(v as usize+1, JValue::Undefined);
            }
            self.values.to_mut()[v as usize] = value;
            return true
        }
        return false
    }


    fn constructor(this:JValue, args:&[JValue]) -> Vec<JValue>{
        Vec::new()
    }


    fn from_(this:JValue, args:&[JValue]) -> Vec<JValue>{
        if args.len() == 0{
            operator::throw(super::Error::newTypeError("Array.from expected at least one argument.").into())
        }
        let mut values = operator::IteratorCollect(args[0]);

        let thisArg = if args.len() > 2{
            args[2]
        } else{
            this
        };

        if args.len() > 1{
            values = values.iter().map(|v|{
                match args[1].call(thisArg, &[*v]){
                    Ok(v) => v,
                    Err(e) => operator::throw(e)
                }
            }).collect::<Vec<JValue>>();
        }

        return values
    }

    fn is_array(this:JValue, arr:Option<JValue>) -> bool{
        if let Some(v) = arr{
            if let Some(o) = v.object(){
                return o.inner.is_array()
            }
        }
        false
    }

    fn of(this:JValue, args:&[JValue]) -> Vec<JValue>{
        args.to_vec()
    }


    fn at(this:Option<&mut Self>, mut idx:i32) -> JValue{
        let this  = check_self(this, "at");
        if idx < 0{
            idx = idx + this.values.len() as i32
        }

        if idx < 0{
            return JValue::Undefined
        }

        if idx < this.values.len() as i32{
            return this.values[idx as usize]
        }
        JValue::Undefined
    }




}

fn check_self<'a>(this:Option<&'a mut Array>, name:&'static str) -> &'a mut Array{
    if let Some(v) = this{
        return v
    }
    operator::throw(super::Error::newTypeError(format!("Array.prototype.{}: require this to be array.", name)))
}

pub unsafe fn init(ctx:&mut VmContext, global:&'static mut JObject){

    let constructor = Function::native(Array::constructor).object().unwrap();
    let proto = JObject::new();

    global.builtin_member("Array", std::ptr::read(&constructor));
    constructor.builtin_member("prototype", JValue::Object(proto));

    constructor.builtin_member("from", Function::native(Array::from_));
    constructor.builtin_member("isArray", Function::native(Array::is_array));
    constructor.builtin_member("of", Function::native(Array::of));

    proto.builtin_member("length", 0i32);
    proto.builtin_member("at", Function::native(Array::at));
}