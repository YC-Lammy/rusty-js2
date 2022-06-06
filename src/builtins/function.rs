use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::RUNTIME;
use crate::utils::BuildNoHasher;
use crate::value::JValue;
use crate::vm::{
    Variable,
    VmContext
};
use crate::bindgen;

use super::JObject;
use super::object::{JObjectInner, JObjectInnerEnum};


#[test]
fn assert_arg_size(){
    unsafe{std::mem::transmute::<&[u8], (usize, usize)>(b"")};
}

pub struct Function{
    captures:Arc<HashMap<u64, Arc<JValue>, BuildNoHasher>>,

    func:Arc<dyn Fn(&mut VmContext, JValue, &[JValue]) -> JValue>,
    is_async:bool,

    mem:Option<*mut u8>
}

impl Function{
    pub fn native<F, Args, T>(f:F) -> JValue where F:Fn<Args, Output = T> +'static, Args:bindgen::Arguments, T:bindgen::Returnable{
        let obj = JObject::new();
        let func = bindgen::bind_function(f);
        Self::from_object(obj, func, false, false)
    }

    pub fn from_object(obj:&'static mut JObject, func:Arc<dyn Fn(&mut VmContext, JValue, &[JValue]) -> JValue>, is_async:bool, is_generator:bool) -> JValue{
        obj.inner = JObjectInnerEnum::Function(Function{
            captures:Arc::new(Default::default()),
            func:func,
            is_async,

            mem:None
        });


        JValue::Object(obj)
    }

    /// call after declaration of function
    pub(crate) fn try_capture(&mut self, vmctx:&'static mut VmContext, names:&[u64]){

        let captures = unsafe{(self.captures.as_ref() as *const _ as *mut HashMap<u64, Arc<JValue>, BuildNoHasher>).as_mut().unwrap()};
        // no need to capture in global context
        if vmctx.parent.is_some() {
            for name in names{
                if let Some(v) = vmctx.capture(*name){
                    captures.insert(*name, v);
                }
            }
        }
    }

    pub(crate) fn new_from_memory(vmctx:&'static mut VmContext, mem:*mut u8, is_async:bool, is_generator:bool){

    }
    
}

unsafe impl Sync for Function{}
unsafe impl Send for Function{}

impl JObjectInner for Function{
    fn call(&mut self, vmctx:&mut VmContext, this:JValue, args:&[JValue]) -> JValue {

        let ctx = vmctx.new_child();
        ctx.attach_captures(self.captures.clone());

        let re = (self.func)(ctx, this, args);

        ctx.done();
        return re;
    }
}

impl Drop for Function{
    fn drop(&mut self) {
        if let Some(mem) = self.mem{
            RUNTIME.with(|runtime|{
                runtime.to_mut().release_compiled_fn(mem);
            })
        }
    }
}