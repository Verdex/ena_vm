
use crate::data::{F64_SIZE, I64_SIZE, PTR_SIZE, OFFSET_SIZE, BOOL_SIZE};

pub trait Byteable<const S:usize> {
    fn to(self) -> [u8; S];
    fn from(input : [u8; S]) -> Self;
}

impl Byteable<F64_SIZE> for f64 {
    fn to(self) -> [u8; F64_SIZE] {
        f64::to_ne_bytes(self)
    }
    fn from(input : [u8; F64_SIZE]) -> Self {
        f64::from_ne_bytes(input)
    }
}

impl Byteable<OFFSET_SIZE> for isize {
    fn to(self) -> [u8; OFFSET_SIZE] {
        isize::to_ne_bytes(self)
    }
    fn from(input : [u8; OFFSET_SIZE]) -> Self {
        isize::from_ne_bytes(input)
    }
}

impl Byteable<I64_SIZE> for i64 {
    fn to(self) -> [u8; I64_SIZE] {
        i64::to_ne_bytes(self)
    }
    fn from(input : [u8; I64_SIZE]) -> Self {
        i64::from_ne_bytes(input)
    }
}

impl Byteable<PTR_SIZE> for usize {
    fn to(self) -> [u8; PTR_SIZE] {
        usize::to_ne_bytes(self)
    }
    fn from(input : [u8; PTR_SIZE]) -> Self {
        usize::from_ne_bytes(input)
    }
}

impl Byteable<BOOL_SIZE> for bool {
    fn to(self) -> [u8; BOOL_SIZE] {
        [self as u8]
    }
    fn from(input : [u8; BOOL_SIZE]) -> Self {
        input[0] != 0
    }
}
