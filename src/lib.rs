#![allow(clippy::len_without_is_empty)]

mod intern;
pub mod search;
pub mod update;

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Header {
    version: [u8; 16],

    providers_len: u32,
    strings_len: u32,
}

unsafe impl Pod for Header {}
unsafe impl Zeroable for Header {}

pub const HEADER_VERSION: [u8; 16] = *b"fcnf format 003\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Provider {
    repo: u32,
    package_name: u32,
    dir: u32,
    bin: u32,
}

unsafe impl Pod for Provider {}
unsafe impl Zeroable for Provider {}
