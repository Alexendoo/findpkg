use crate::{phf, Header, Provider, Span, HEADER_VERSION};
use anyhow::{ensure, Result};
use bytemuck::{cast_slice, from_bytes, Pod};
use std::fmt::Write;
use std::{mem, str};

pub enum Entry {
    Found(String),
    NotFound,
}

fn split_cast<T: Pod>(slice: &[u8], mid: u32) -> (&[T], &[u8]) {
    let (bytes, rest) = slice.split_at(mid as usize);
    (cast_slice(bytes), rest)
}

pub fn search(command: &str, db: &[u8]) -> Result<Entry> {
    let (header_bytes, rest) = db.split_at(mem::size_of::<Header>());
    let header: Header = *from_bytes(header_bytes);

    ensure!(
        header.version == HEADER_VERSION,
        "unknown header version {:?}",
        String::from_utf8_lossy(&header.version),
    );

    let (providers, rest) = split_cast::<Provider>(rest, header.providers_len);
    let (disps, rest) = split_cast::<phf::Disp>(rest, header.disps_len);
    let (table, string_buf) = split_cast::<Span>(rest, header.table_len);

    let hashes = phf::hash(command, header.hash_key);
    let index = phf::get_index(&hashes, disps, table.len());

    let providers_span = table[index as usize];
    let bin_providers = providers_span.get(providers);

    if bin_providers[0].bin.get(string_buf) != command.as_bytes() {
        // println!("Command not found: {}", command);
        return Ok(Entry::NotFound);
    }

    let max_len = bin_providers
        .iter()
        .map(|prov| prov.repo.len() + prov.package_name.len())
        .max()
        .unwrap();

    let mut out = String::new();

    for provider in bin_providers {
        let repo = provider.repo.get_str(string_buf);
        let package_name = provider.package_name.get_str(string_buf);
        let dir = provider.dir.get_str(string_buf);
        let bin = provider.bin.get_str(string_buf);

        writeln!(
            out,
            "{}/{:padding$}\t/{}{}",
            repo,
            package_name,
            dir,
            bin,
            padding = max_len - repo.len(),
        )?;
    }

    Ok(Entry::Found(out))
}
