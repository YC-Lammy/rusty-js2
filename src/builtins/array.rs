


use std::sync::Arc;

use crate::{value::JValue, vm::VmContext};
use crate::operator;


use super::object::{
    JObject,
    JObjectInner
};
use super::function::Function;

pub struct Array{
    pub(crate) values:Vec<JValue>
}

impl Array{
    pub fn new(values:&[JValue]) -> Arc<dyn JObjectInner>{
        return Arc::new(Array{
            values:values.to_vec()
        })
    }

    pub unsafe fn new_raw(argv:*mut JValue, argc:i64, spread:bool) -> Arc<dyn JObjectInner>{
        let mut args = std::mem::transmute::<_, &[JValue]>((argv, argc as usize));
        if spread{
            let mut v = args.to_vec();
            v.extend(operator::IteratorCollect(args[args.len() -1]));
            return Self::new(&v)
        };
        Self::new(args)
    }
    
    fn constructor(vmctx:&mut VmContext, this:JValue, args:&[JValue]) -> JValue{
        JValue::Undefined
    }
}

impl JObjectInner for Array{
    fn set(&mut self, name:&str, value:JValue) -> bool {
        if let Ok(mut i) = name.parse::<i64>(){
            if i < 0{
                i = self.values.len() as i64 - i;
            }

            if i < 0{
                return false
            }

            if i as usize >= self.values.len(){
                self.values.resize(i as usize +1, JValue::Undefined);
            }
            self.values[i as usize] = value;
            return true
        } else{
            return false
        }
    }

    fn get(&mut self, name:&str) -> Option<JValue> {
        if let Ok(mut i) = name.parse::<i64>(){
            if i < 0{
                i = self.values.len() as i64 - i;
            }
            if i < 0{
                return None
            }

            if i as usize >= self.values.len(){
                self.values.resize(i as usize +1, JValue::Undefined);
            }
            return Some(self.values[i as usize])
        } else{
            return None
        }
    }
}

pub unsafe fn init(ctx:&mut VmContext, global:&'static mut JObject){

    let constructor = JObject::fromInner(Function::native(Array::constructor));
    let proto = JObject::new();

    global.builtin_member("Array", std::ptr::read(&constructor));
    constructor.builtin_member("prototype", proto);

    constructor.builtin_member("from", |vmctx:&mut VmContext, this, args:&[JValue]|{
        if args.len() == 0{
            operator::throw(super::error::Error::newTypeError("Array.from expected at least one argument.").into())
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

        return JObject::fromInner(Arc::new(Array{
            values
        })).into()
    });

    constructor.builtin_member("isArray", |vmctx:&mut VmContext, this, args:&[JValue]|{
        if args.len() > 0{
            if let Some(o) = args[0].object(){
                if let Some(inner) = &o.inner{
                    return JValue::Boolean(inner.is_array())
                }
            }
        }
        JValue::Boolean(false)
    });

    
}