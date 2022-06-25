use super::DataMarker;

use crate::builtins::JObject;

const OBJECT_SIZE:usize = std::mem::size_of::<MarkedObject>();
const PAGE_SIZE:usize = OBJECT_SIZE * 1024;

#[repr(C)]
pub struct MarkedObject{
    pub(crate) mark:DataMarker,
    pub(crate) object:JObject
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Block{
    mark:DataMarker,
    next:*mut Block,
}

pub struct Allocator{
    allocations:Vec<*mut Block>,

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

    pub fn deallocate(&mut self, obj:&'static mut JObject){
        unsafe{
            let o = (obj as *mut JObject).sub(std::mem::size_of::<MarkedObject>() - std::mem::size_of::<JObject>()) as *mut Block;
            (*o).mark = DataMarker::NotAllocated;
            (*o).next = self.next;
            self.next = o;
        }
    } 

    pub fn extend(&mut self){
        unsafe{
            let page = std::alloc::alloc(std::alloc::Layout::new::<[u8;PAGE_SIZE]>());

            for i in 0..PAGE_SIZE/OBJECT_SIZE{

                let block = page.add(OBJECT_SIZE*i) as *mut Block;

                (*block).next = self.next;
                (*block).mark = DataMarker::NotAllocated;

                self.next = block;
            }
        }
    }

    pub unsafe fn GarbageCollect(&mut self){
        for i in &self.allocations{
            let mut next = *i;
            loop{
                if (next as usize) - *i as usize >= PAGE_SIZE{
                    break;
                }

                match (*next).mark {
                    DataMarker::Old => {
                        (*next).mark = DataMarker::NotUse;
                    },
                    DataMarker::NotUse => {
                        let obj = next as *mut MarkedObject;
                        drop(std::ptr::read(obj).object);

                        (*next).mark = DataMarker::NotAllocated;
                        (*next).next = self.next;
                        self.next = next;
                    },
                    DataMarker::InUse => {
                        (*next).mark = DataMarker::Old;
                    }
                    DataMarker::KeepAlive | 
                    DataMarker::NotAllocated => {},
                }

                next = next.add(std::mem::size_of::<MarkedObject>());
            }
        }
    }
}

