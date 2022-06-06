use super::DataMarker;

use crate::builtins::JObject;

pub struct MarkedObject{
    pub(crate) mark:DataMarker,
    pub(crate) object:JObject
}

#[derive(Clone, Copy)]
pub struct Block{
    mark:DataMarker,
    next:*mut Block,
}

pub struct Allocator{
    allocations:Vec<*mut u8>,

    next:*mut Block
}

impl Allocator{
    pub fn allocate(&mut self) -> &'static mut JObject{
        if self.next as usize == 0{
            self.extend();
        }
        let next = self.next;
        self.next = unsafe{(*next).next};
        return unsafe{(next as *mut JObject).as_mut().unwrap()}
    }

    pub fn extend(&mut self){

    }
}