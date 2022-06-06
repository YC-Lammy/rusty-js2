pub mod object_allocator;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DataMarker{
    NotAllocated,
    KeepAlive,
    InUse,
    Old,
    NotUse
}