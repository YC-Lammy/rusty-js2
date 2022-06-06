
use parking_lot::Mutex;

use crate::heap::{
    SlabAllocator, Slab, DataMarker
};


pub(crate) static STRING_ALLOCATOR:Mutex<SlabAllocator> = Mutex::new(SlabAllocator::new());

