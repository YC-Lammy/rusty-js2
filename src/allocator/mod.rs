pub mod object_allocator;
pub mod string_allocator;
pub mod heap;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DataMarker{
    NotAllocated,
    KeepAlive,
    InUse,
    Old,
    NotUse
}