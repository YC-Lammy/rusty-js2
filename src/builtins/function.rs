use std::collections::HashMap;
use std::sync::Arc;

use crate::util::BuildNoHasher;
use crate::value::JValue;
use crate::vm::{
    Variable,
    VmContext
};

use super::object::JObjectInner;


#[test]
fn assert_arg_size(){
    unsafe{std::mem::transmute::<&[u8], (usize, usize)>(b"")};
}

pub struct Function{
    captures:Arc<HashMap<u64, Arc<JValue>, BuildNoHasher>>,

    func:Arc<dyn Fn(&mut VmContext, JValue, &[JValue]) -> JValue>,
}

impl Function{
    pub fn native<F>(f:F) -> Arc<dyn JObjectInner> where F:Fn(&mut VmContext, JValue, &[JValue]) -> JValue + 'static{
        return Arc::new(Function{
            captures:Arc::new(Default::default()),
            func:Arc::new(f)
        })
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