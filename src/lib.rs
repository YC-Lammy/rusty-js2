#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

mod jit;
mod interpretor;
mod operator;
mod value;
mod builtins;
mod heap;
mod string_allocator;

pub mod runtime;
mod vm;
mod util;
mod error;