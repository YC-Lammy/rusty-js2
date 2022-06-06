use std::alloc::{
    Layout,
    alloc_zeroed,
    dealloc,
    alloc
};
use std::ptr::null_mut;
use std::mem::MaybeUninit;
use std::collections::{HashSet, HashMap};
use std::marker::PhantomData;

use parking_lot::Mutex;
use parking_lot::RawMutex;

use crate::runtime::RUNTIME;

#[repr(u8)]
pub enum DataMarker{
    NotAllocated,
    KeepAlive,
    InUse,
    Old,
    NotUse
}

#[repr(C)]
pub struct MarkedObject{
    
}
pub fn malloc<T>() -> &'static mut T{
    unsafe{RUNTIME.with(|runtime|{
        runtime.to_mut().allocator.alloc(Layout::new::<T>()) as *mut T
    }).as_mut().unwrap()}
}

pub fn free<T>(ptr:&'static mut T){
    unsafe{RUNTIME.with(|runtime|{
        runtime.to_mut().allocator.dealloc(ptr as *mut T as *mut u8, Layout::new::<T>());
    })
    }
}

unsafe impl Sync for SlabAllocator{}
unsafe impl Send for SlabAllocator{}

lazy_static::lazy_static!{
    pub(crate) static ref ALLOCATORS:HashMap<i32, Mutex<SlabAllocator>> = HashMap::new();
}

pub fn SlabAlloc(id:i32, size:i64) -> *mut u8{
    ALLOCATORS.get(&id).unwrap().lock().alloc(Layout::array::<u8>(size as usize).unwrap())
}

pub fn SlabFree(id:i32, addr:*mut u8, size:i64){
    ALLOCATORS.get(&id).unwrap().lock().dealloc(addr, Layout::array::<u8>(size as usize).unwrap())
}

pub(crate) struct SlabAllocator{
    pub slab16:Slab<16>,
    pub slab32:Slab<32>,
    pub slab64:Slab<64>,
    pub slab128:Slab<128>,
    pub slab256:Slab<256>,
    pub slab512:Slab<512>,
    pub slab1024:Slab<1024>,
    pub slab2048:Slab<2048>,
    pub slab4096:Slab<4096>,
    pub others:Option<rustc_hash::FxHashSet<*mut u8>>,
}

impl SlabAllocator{
    pub const fn new() -> Self{
        SlabAllocator{
            slab16:Slab::<16>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab32:Slab::<32>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab64:Slab::<64>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab128:Slab::<128>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab256:Slab::<256>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab512:Slab::<512>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab1024:Slab::<1024>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab2048:Slab::<2048>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            slab4096:Slab::<4096>{
                allocates:Vec::new(),
                freelist:null_mut(),
                phantom:PhantomData
            },
            others:None
        }
    }
    #[inline]
    pub fn alloc(&mut self, layout:Layout) -> *mut u8{
        if layout.size() > 4096 {
            let ptr = unsafe{alloc(layout)};
            match &mut self.others{
                Some(h) => {
                    h.insert(ptr);
                },
                None => {
                    let mut h = HashSet::default();
                    h.insert(ptr);
                    self.others = Some(h);
                }
            };
            ptr
        } else if layout.size() <= 16 && layout.align() <= 16{
            self.slab16.alloc()
        } else if layout.size() <= 32 && layout.align() <= 32{
            self.slab32.alloc()
        } else if layout.size() <= 64 && layout.align() <= 64 {
            self.slab64.alloc()
        } else if layout.size() <= 128 && layout.align() <= 128 {
            self.slab128.alloc()
        } else if layout.size() <= 256 && layout.align() <= 256 {
            self.slab256.alloc()
        } else if layout.size() <= 512 && layout.align() <= 512 {
            self.slab512.alloc()
        } else if layout.size() <= 1024 && layout.align() <= 1024 {
            self.slab1024.alloc()
        } else if layout.size() <= 2048 && layout.align() <= 2048 {
            self.slab2048.alloc()
        } else {
            self.slab4096.alloc()
        }
    }

    #[inline]
    pub fn dealloc(&mut self, ptr:*mut u8, layout:Layout){
        if layout.size() > 4096 {
            match &mut self.others{
                Some(h) => h.remove(&ptr),
                None => unreachable!("pointer register does not exit, value not alloc by Allocator.")
            };
            unsafe{dealloc(ptr, layout)};
        } else if layout.size() <= 16 && layout.align() <= 16{
            self.slab16.dealloc(ptr)
        } else if layout.size() <= 32 && layout.align() <= 32{
            self.slab32.dealloc(ptr)
        } else if layout.size() <= 64 && layout.align() <= 64 {
            self.slab64.dealloc(ptr)
        } else if layout.size() <= 128 && layout.align() <= 128 {
            self.slab128.dealloc(ptr)
        } else if layout.size() <= 256 && layout.align() <= 256 {
            self.slab256.dealloc(ptr)
        } else if layout.size() <= 512 && layout.align() <= 512 {
            self.slab512.dealloc(ptr)
        } else if layout.size() <= 1024 && layout.align() <= 1024 {
            self.slab1024.dealloc(ptr)
        } else if layout.size() <= 2048 && layout.align() <= 2048 {
            self.slab2048.dealloc(ptr)
        } else {
            self.slab4096.dealloc(ptr)
        }
    }
}

#[repr(C)]
pub struct SlabEntry{
    state:DataMarker,
    next:*mut SlabEntry
}

pub(crate) struct Slab<const BLOCK_SIZE:usize>{
    pub(crate) allocates:Vec<*mut u8>,
    freelist:*mut SlabEntry,

    phantom:PhantomData<[u8;BLOCK_SIZE]>
}

impl<const BLOCK_SIZE:usize> Slab<BLOCK_SIZE>{
    
    #[inline]
    fn alloc(&mut self) -> *mut u8{
        if self.freelist as usize == 0{
            self.grow(4096 * 4);
        };
        let next = unsafe{&(*self.freelist)}.next;
        let a = self.freelist;
        self.freelist = next;
        a as *mut u8
    }

    #[inline]
    fn dealloc(&mut self, ptr:*mut u8){
        let next = self.freelist;
        let entry = ptr as *mut SlabEntry;
        unsafe{(*entry).state = DataMarker::NotAllocated};
        unsafe{(*entry).next = next};
        self.freelist = entry;
    }

    /// grow in bytes
    #[inline]
    fn grow(&mut self, size:usize) -> *mut u8{
        let page = unsafe{alloc_zeroed(Layout::from_size_align(size, 4096).unwrap())};

        for i in 0..size/BLOCK_SIZE{
            let entry = unsafe{page.add(i*BLOCK_SIZE)} as *mut SlabEntry;
            unsafe{(*entry).next = self.freelist};
            self.freelist = entry;
        };

        self.allocates.push(page);

        page
    }
}