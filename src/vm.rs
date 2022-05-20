use std::{sync::Arc, collections::HashMap};

use rustc_hash::FxHashMap;

use crate::operator;
use crate::runtime::Runtime;
use crate::value::JValue;
use crate::util::{
    BuildNoHasher,
    NoHasher
};

#[derive(Clone)]
pub enum Variable{
    Let(JValue),
    Var(JValue),
    Const(JValue),

    Captured(Arc<JValue>)
}

pub struct VmContext{
    pub(crate) runtime:&'static mut Runtime,
    pub(crate) parent:Option<&'static mut VmContext>,
    pub(crate) variables:HashMap<u64, Variable, BuildNoHasher>,

    pub(crate) captures:Option<Arc< HashMap<u64, Arc<JValue>, BuildNoHasher> >>
}

impl VmContext{
    pub fn new_child(&mut self) -> &'static mut Self{
        Box::leak(Box::new(Self{
            runtime:unsafe{std::mem::transmute_copy(&self.runtime)},
            parent:Some(unsafe{std::mem::transmute(self)}),
            variables:Default::default(),
            captures:None,
        }))
    }

    pub fn done(&mut self){
        unsafe{std::ptr::drop_in_place(self)};
    }

    pub fn attach_captures(&mut self, c:Arc<HashMap<u64, Arc<JValue>, BuildNoHasher>>){
        self.captures = Some(c)
    }

    pub fn capture(&mut self, name:u64) -> Option<Arc<JValue>>{
        if let Some(variable) = self.variables.get_mut(&name){
            match variable{
                Variable::Captured(c) => {
                    Some(c.clone())
                },
                Variable::Const(c) => {
                    let a = Arc::new(*c);
                    *variable = Variable::Captured(a.clone());
                    Some(a.clone())
                },
                Variable::Let(l) => {
                    let a = Arc::new(*l);
                    *variable = Variable::Captured(a.clone());
                    Some(a.clone())
                },
                Variable::Var(c) => {
                    let a = Arc::new(*c);
                    *variable = Variable::Captured(a.clone());
                    Some(a.clone())
                }
            }
        } else{
            if let Some(o) = &self.captures{
                if let Some(v) = o.get(&name){
                    return Some(v.clone())
                };
            };

            if let Some(p) = &mut self.parent{
                p.capture(name)
            } else{
                None
            }
        }
    }

    /// return true if present, else false
    pub fn get_variable_raw(&self, name:u64) -> (JValue, bool){
        if let Some(v) = self.variables.get(&name){
            (match v{
                Variable::Captured(c) => **c,
                Variable::Const(v) => *v,
                Variable::Let(v) => *v,
                Variable::Var(v) => *v
            }, true)
        }else{
            if let Some(o) = &self.captures{
                if let Some(v) = o.get(&name){
                    return (**v, true)
                };
            };
            if let Some(p) = &self.parent{
                p.get_variable_raw(name)
            } else{
                // todo: throw reference error
                (JValue::Undefined, false)
            }
        }
    }

    pub fn get_variable(&self, name:u64) -> JValue{
        let (re, ok) = self.get_variable_raw(name);
        if !ok{
            operator::throw(re)
        }
        re
    }

    pub fn set_variable(&mut self, name:u64, value:JValue){
        if let Some(v) = self.variables.get_mut(&name){
            match v{
                Variable::Captured(c) => unsafe{
                    (c.as_ref() as *const JValue as *mut JValue).write(value)
                },
                Variable::Const(v) => *v = value,
                Variable::Let(v) => *v = value,
                Variable::Var(v) => *v = value
            }
        } else{
            if let Some(o) = &self.captures{
                if let Some(v) = o.get(&name){
                    unsafe{
                        (v.as_ref() as *const JValue as *mut JValue).write(value)
                    }
                    return;
                };
            };

            if let Some(p) = &mut self.parent{
                p.set_variable(name, value);
            }
        }
    }

    pub fn get_variable_str(&mut self, name:&str) -> JValue{
        let s = self.runtime.to_mut().new_variable_name(name);
        self.get_variable(s as u64)
    }

    pub fn set_variable_str(&mut self, name:&str, value:JValue){
        let s = self.runtime.new_variable_name(name);
        self.set_variable(s as u64, value)
    }
}