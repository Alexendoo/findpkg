use crate::intern::Span;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Result};
use bytemuck::{cast_slice, from_bytes};
use fst::Map;
use memmap::Mmap;
use std::fs::File;
use std::mem;
use std::str::{self, Utf8Error};

fn get_str(buf: &[u8], span: Span) -> Result<&str, Utf8Error> {
    let slice = &buf[span.start as usize..span.end as usize];

    str::from_utf8(slice)
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

    let (provider_bytes, rest) = rest.split_at(header.providers_len as usize);
    let providers: &[Provider] = cast_slice(provider_bytes);

    let (strings, fst_bytes) = rest.split_at(header.strings_len as usize);

    let map = Map::new(fst_bytes)?;
    let val = map.get(command).unwrap();

    let start = (val & ((1 << 32) - 1)) as usize;
    let end = start + (val >> 32) as usize;

    for provider in &providers[start..end] {
        let repo = get_str(strings, provider.repo)?;
        let package_name = get_str(strings, provider.package_name)?;
        let dir = get_str(strings, provider.dir)?;

        println!("{}/{}\t/{}{}", repo, package_name, dir, command);
    }

    // let header_size = bincode::serialized_size(&header)? as usize;
    // let fst_end = header_size + header.fst_len as usize;
    // let fst_bytes = &mmap[header_size..fst_end];

    // let map = Map::new(fst_bytes)?;z
    // let packages = &mmap[fst_end..];

    // let lev = Levenshtein::new(command, 1)?;
    // let mut stream = map.search(lev).into_stream();

    // while let Some((bin, index)) = stream.next() {
    //     let bin = String::from_utf8_lossy(bin);
    //     let providers: Providers = bincode::deserialize(&packages[index as usize..])?;

    //     eprintln!("(bin, index) = {:?}", (bin, providers));
    // }

    Ok(())
}
