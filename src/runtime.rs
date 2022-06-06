
use std::alloc::Layout;
use std::cell::Cell;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::ops::*;
use std::sync::Arc;

use cranelift::codegen::Context;
use cranelift::prelude::Signature;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::FuncId;
use cranelift_module::Module;
use cranelift_module::DataContext;

use cranelift::prelude::{
    AbiParam, types
};


use parking_lot::RwLock;
use string_interner::Symbol;
use string_interner::{
    StringInterner,
    DefaultBackend,
    symbol::SymbolUsize,
};
use swc_ecma_ast::ModuleItem;

use crate::builtins;
use crate::builtins::JObject;
use crate::error::Error;
use crate::jit::builder::BuilderContext;
use crate::parse::parse_ecma;
use crate::prelude::OwnedValue;
use crate::value::JValue;
use crate::vm::VmContext;
use crate::heap::SlabAllocator;

thread_local!{
    pub(crate) static RUNTIME:&'static mut Runtime = unsafe{std::mem::zeroed()};
}

macro_rules! declare_fn {
    ($self:ident, $module:ident, $call_conv:ident, $name:expr ; $( $params:ident ),* => $( $returns:ident ),*) => {
        $self.builtin_functions.insert($name, $module.declare_function($name, cranelift_module::Linkage::Import, &Signature{
            params:vec![$(AbiParam::new(types::$params)),*],
            returns:vec![$(AbiParam::new(types::$returns)),*],
            call_conv:$call_conv
        }).expect("error while initializing bultin functions"));
    };
}


#[derive(Clone, Copy)]
pub(crate) struct compiled_func{
    rc:usize,
    size:usize,
}

pub struct Runtime{

    pub(crate) allocator:SlabAllocator,

    pub(crate) module:Arc<JITModule>,

    pub(crate) ctx:&'static mut Context,

    pub(crate) context:VmContext,

    pub(crate) global:&'static mut JObject,

    pub(crate) variable_names:StringInterner<DefaultBackend<SymbolUsize>>,

    pub (crate) builtin_functions:HashMap<&'static str, FuncId>,

    pub(crate) compiled_functions:HashMap<*mut u8, compiled_func>
}

unsafe impl Send for Runtime{}
unsafe impl Sync for Runtime{}

impl Runtime{

    pub fn new() -> Arc<Self>{
        
        
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.hotswap(true);


        init_builder(&mut builder);

        let module = Arc::new(JITModule::new(builder));

        let mut runtime = Arc::new(Self{

            allocator:SlabAllocator::new(),
            context:VmContext::new(),
            variable_names:StringInterner::new(),

            module:module.clone(),
            ctx:Box::leak(Box::new(module.make_context())),
            global:JObject::new(),

            builtin_functions:HashMap::new(),
            compiled_functions:Default::default(),
        });

        let r = runtime.to_mut();
        r.context.runtime = unsafe{std::mem::transmute_copy(&r)};


        return runtime
    }

    fn init(&self){
        RUNTIME.with(|runtime|unsafe{
            (runtime as *const _ as *mut &'static mut Runtime).write(std::mem::transmute_copy(&self))
        });
    }

    pub(crate)fn to_mut(&self) -> &'static mut Self{
        unsafe{std::mem::transmute_copy(&self)}
    }

    pub(crate)fn new_variable_name<T>(&mut self, name:T) -> usize where T:AsRef<str> {
        self.variable_names.get_or_intern(name).to_usize()
    }

    pub(crate) fn new_compiled_fn(&mut self, mem:*mut u8, size:usize){
        self.compiled_functions.insert(mem, compiled_func{
            rc:0,
            size,
        });
    }
    
    pub(crate) fn release_compiled_fn(&mut self, mem:*mut u8){
        if let Some(f) = self.compiled_functions.get(&mem){
            unsafe{std::alloc::dealloc(mem, Layout::array::<u8>(f.size).unwrap())};
        }
    }



    pub fn exec(self:Arc<Self>,filename:&str, script:&str) -> Result<OwnedValue, Error>{
        self.init();

        let builder_ctx = BuilderContext::new(self.clone(), self.module.clone(), self.to_mut().ctx);

        let module = parse_ecma(filename, script)?;

        for item in &module.body{
            match item{
                ModuleItem::Stmt(s) => {

                },
                _ => todo!()
            }
        };

        let info = self.to_mut().ctx.compile(self.module.isa());
        let info = match info{
            Ok(v) => v,
            Err(e) => {
                self.to_mut().ctx.clear();
                return Err(Error::CodegenError(Arc::new(e)))
            }
        };

        let mem = unsafe{std::alloc::alloc(std::alloc::Layout::array::<u8>(info.total_size as usize).unwrap())};
        unsafe{self.ctx.emit_to_memory(mem)};

        self.to_mut().ctx.clear();

        let func:fn(*mut VmContext, JValue, *mut JValue, i64) -> JValue = unsafe{std::mem::transmute(mem)};
        let v = func(&mut self.to_mut().context, JValue::Object(self.to_mut().global), 1 as _, 0);

        v.keep_alive(true);

        unsafe{
            std::alloc::dealloc(mem, std::alloc::Layout::array::<u8>(info.total_size as usize).unwrap());
        }

        // return the value
        Ok(OwnedValue{
            value:crate::prelude::JValue { 
                value: v, 
                marker: std::marker::PhantomData 
            }
        })
    }
    


    fn init_functions(&mut self){

        let default_call_conv = self.module.target_config().default_call_conv;

        let module = unsafe{(self.module.as_ref() as *const _ as *mut JITModule).as_mut().unwrap()};

        declare_fn!(self, module, default_call_conv, "construct"; I128 => I128);
        declare_fn!(self, module, default_call_conv, "call"; I64, I128, I64, I64, B8 => I128, B8);
    }
    
}



fn init_builder(builder:&mut JITBuilder){
    //builder.symbol("alloc", heap::SlabAlloc as *const u8);
    //builder.symbol("dealloc", heap::SlabFree as *const u8);

    builder.symbol("construct", JValue::new as *const u8);
    builder.symbol("call", JValue::call_raw as *const u8);
    builder.symbol("add", JValue::add as *const u8);
    builder.symbol("sub", JValue::sub as *const u8);
    builder.symbol("div", JValue::div as *const u8);
    builder.symbol("mul", JValue::mul as *const u8);

    builder.symbol("function_new", builtins::Function::new_from_memory as *const u8);
}