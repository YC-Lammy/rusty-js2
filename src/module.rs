use crate::{value::JValue, vm::VmContext};



pub struct Module{
    default_export:JValue,
    context:VmContext,

    
}

impl Module{
    pub(crate) fn set_default(&mut self, value:JValue){
        self.default_export = value;
    }

    pub(crate) fn set_named_export(&mut self, name:u64, value:JValue){

    }
}