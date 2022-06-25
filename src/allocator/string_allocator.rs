
use std::marker::PhantomData;

use super::DataMarker;

#[repr(C)]
pub struct SlabEntry{
    marker:DataMarker,
    next:*mut SlabEntry
}

pub(crate) struct Slab<const SIZE:usize>{
    pub(crate) allocates:Vec<*mut [SlabEntry;SIZE]>,
    freelist:*mut SlabEntry,
}

#[repr(C)]
pub struct StringBlock<const SIZE:usize>{
    marker:DataMarker,
    value:[u8;SIZE]
}

pub struct StringAllocator{
    slab32:Slab<32>,
    slab64:Slab<64>,
    slab128:Slab<128>,
    slab256:Slab<256>,
    slab512:Slab<512>,
    slab1024:Slab<1024>,
    slab2048:Slab<2048>,
    
}


impl<const SIZE:usize> Slab<SIZE>{
    pub fn new() -> Self{
        Self { 
            allocates: Vec::new(), 
            freelist: 0 as *mut SlabEntry 
        }
    }
}