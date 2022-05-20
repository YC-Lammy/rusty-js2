
use parking_lot::Mutex;

use crate::heap::SlabAllocator;


pub(crate) static STRING_ALLOCATOR:Mutex<SlabAllocator> = Mutex::new(SlabAllocator::new());
