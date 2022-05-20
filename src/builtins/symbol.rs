use std::sync::atomic::{AtomicUsize, AtomicU32};
use std::ops::Deref;

use string_interner::{
    StringInterner,
    symbol::SymbolU32,
};

use parking_lot::RwLock;

use crate::value::JValue;

lazy_static::lazy_static!{
    static ref INTERNER:StringInterner = StringInterner::new();

    pub static ref Iterator:JValue = Symbol::new("iterator");
}

static SYMBOL_COUNT:AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy)]
pub struct Symbol{
    pub(crate) id:u32,
    pub(crate) intern:SymbolU32,
}

impl Symbol{
    pub fn new(s:&str) -> JValue{
        let id = SYMBOL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let i = unsafe{(INTERNER.deref() as *const _ as *mut StringInterner).as_mut().unwrap()};
        let intern = i.get_or_intern(s);
        return JValue::Symbol(Symbol{
            id,
            intern
        })
    }
}

impl Deref for Symbol{
    type Target = str;
    fn deref(&self) -> &Self::Target {
        INTERNER.resolve(self.intern).unwrap()
    }
}

impl AsRef<str> for Symbol{
    fn as_ref(&self) -> &str {
        INTERNER.resolve(self.intern).unwrap()
    }
}