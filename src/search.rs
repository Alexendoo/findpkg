use crate::intern::Span;
use crate::phf;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Result};
use bytemuck::{cast_slice, from_bytes, Pod};
use fst::Map;
use memmap::Mmap;
use std::fs::File;
use std::{mem, str};

fn split_cast<T: Pod>(slice: &[u8], mid: u32) -> (&[T], &[u8]) {
    let (bytes, rest) = slice.split_at(mid as usize);
    (cast_slice(bytes), rest)
}

pub fn search(command: &str) -> Result<()> {
    let db_file = File::open("./out.db")?;
    let mmap = unsafe { Mmap::map(&db_file)? };

    let (header_bytes, rest) = mmap.split_at(mem::size_of::<Header>());
    let header: Header = *from_bytes(header_bytes);

    ensure!(
        header.version == HEADER_VERSION,
        "unknown header version {:?}",
        header.version
    );

    let (providers, rest) = split_cast::<Provider>(rest, header.providers_len);
    let (disps, rest) = split_cast::<phf::Disp>(rest, header.disps_len);
    let (table, string_buf) = split_cast::<Span>(rest, header.table_len);

    let hashes = phf::hash(command, header.hash_key);
    let index = phf::get_index(&hashes, disps, table.len());

    let providers_span = table[index as usize];
    // let bin_providers

    Ok(())
}
