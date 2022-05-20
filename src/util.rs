use std::hash::{
    Hash, Hasher, BuildHasher, BuildHasherDefault
};

pub struct NoHasher{
    n:u64,
}

pub type BuildNoHasher = BuildHasherDefault<NoHasher>;

impl Hasher for NoHasher{
    fn write(&mut self, bytes: &[u8]) {
        let mut h = rustc_hash::FxHasher::default();
        h.write(bytes);
        self.n = h.finish();
    }

    fn write_i128(&mut self, i: i128) {
        let mut h = rustc_hash::FxHasher::default();
        h.write_i128(i);
        self.n = h.finish();
    }

    fn write_i16(&mut self, i: i16) {
        self.n = i as u64
    }

    fn write_i32(&mut self, i: i32) {
        self.n = i as u64
    }

    fn write_i64(&mut self, i: i64) {
        self.n = i as u64
    }

    fn write_i8(&mut self, i: i8) {
        self.n = i as u64
    }

    fn write_isize(&mut self, i: isize) {
        self.n = i as u64
    }

    fn write_u128(&mut self, i: u128) {
        let mut h = rustc_hash::FxHasher::default();
        h.write_u128(i);
        self.n = h.finish();
    }

    fn write_u16(&mut self, i: u16) {
        self.n = i as u64
    }

    fn write_u32(&mut self, i: u32) {
        self.n = i as u64
    }

    fn write_u64(&mut self, i: u64) {
        self.n = i
    }

    fn write_u8(&mut self, i: u8) {
        self.n = i as u64
    }

    fn write_usize(&mut self, i: usize) {
        self.n = i as u64
    }

    fn finish(&self) -> u64 {
        return self.n
    }
}

impl Default for NoHasher{
    fn default() -> Self {
        return Self { n: 0 }
    }
}