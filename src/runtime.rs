
use std::cell::Cell;
use std::mem::MaybeUninit;
use std::ops::*;

use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
use cranelift_module::DataContext;


use string_interner::Symbol;
use string_interner::{
    StringInterner,
    DefaultBackend,
    symbol::SymbolUsize,
};

use crate::value::JValue;
use crate::vm::VmContext;
use crate::heap::SlabAllocator;

thread_local!{
    pub(crate) static RUNTIME:&'static mut Runtime = unsafe{std::mem::zeroed()};
}

pub struct Runtime{

    pub(crate) allocator:SlabAllocator,

    pub(crate) context:VmContext,

    pub(crate) variable_names:StringInterner<DefaultBackend<SymbolUsize>>
}

impl Runtime{
    fn init(&mut self){
        RUNTIME.with(|runtime|unsafe{
            (runtime as *const _ as *mut &mut Runtime).write(self)
        });
    }

    pub(crate)fn to_mut(&self) -> &'static mut Self{
        unsafe{std::mem::transmute_copy(&self)}
    }

    pub(crate)fn new_variable_name<T>(&mut self, name:T) -> usize where T:AsRef<str> {
        self.variable_names.get_or_intern(name).to_usize()
    }
}

pub fn newRuntime(){
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

    builder.hotswap(true);
    init_symbols(&mut builder);

    let mut module = JITModule::new(builder);
    let mut context = module.make_context();

    let mut heapContext = DataContext::new();

    let data = module.declare_data("Heap", cranelift_module::Linkage::Export, true, false).unwrap();
    module.define_data(data, &heapContext);
}

pub fn init_symbols(builder:&mut JITBuilder){
    //builder.symbol("alloc", heap::SlabAlloc as *const u8);
    //builder.symbol("dealloc", heap::SlabFree as *const u8);

    builder.symbol("new", JValue::new as *const u8);
    builder.symbol("call", JValue::call_raw as *const u8);
    builder.symbol("add", JValue::add as *const u8);
    builder.symbol("sub", JValue::sub as *const u8);
    builder.symbol("div", JValue::div as *const u8);
    builder.symbol("mul", JValue::mul as *const u8);
}