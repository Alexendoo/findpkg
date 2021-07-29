pub mod index;
mod intern;
mod phf;
pub mod search;

use bytemuck::{Pod, Zeroable};
use std::str;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    version: [u8; 16],

    hash_key: u64,

    providers_len: u32,
    disps_len: u32,
    table_len: u32,
    strings_len: u32,
}

unsafe impl Pod for Header {}
unsafe impl Zeroable for Header {}

pub const HEADER_VERSION: [u8; 16] = *b"fcnf format 001\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

unsafe impl Pod for Span {}
unsafe impl Zeroable for Span {}

impl Span {
    pub fn get<T>(self, slice: &[T]) -> &[T] {
        &slice[self.start as usize..self.end as usize]
    }

    pub fn get_str(self, bytes: &[u8]) -> &str {
        str::from_utf8(self.get(bytes)).unwrap()
    }

    pub fn len(self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Provider {
    repo: Span,
    package_name: Span,
    dir: Span,
    bin: Span,
}

unsafe impl Pod for Provider {}
unsafe impl Zeroable for Provider {}
