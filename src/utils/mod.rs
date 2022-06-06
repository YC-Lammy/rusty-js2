pub mod nohasher;

pub use nohasher::*;

pub(crate) trait ToMutable{
    fn to_mut(&self) -> &mut Self{
        unsafe{(self as *const Self as *mut Self).as_mut().unwrap()}
    }
}

impl<T> ToMutable for T{}
