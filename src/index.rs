use crate::intern::Interner;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{Context, Result};
use bytemuck::{bytes_of, cast_slice, Pod};
use std::convert::TryInto;
use std::io::prelude::*;

fn byte_len<T: Pod>(slice: &[T]) -> u32 {
    cast_slice::<T, u8>(slice).len().try_into().unwrap()
}

pub fn index(mut list: impl BufRead, mut db: impl Write) -> Result<()> {
    let mut strings = Interner::new();
    let mut providers = Vec::new();

    let mut line = String::new();

    loop {
        line.clear();
        if list.read_line(&mut line).context("Failed to read line")? == 0 {
            break;
        }
        let mut parts = line.rsplit('\0');
        let mut pop = || parts.next().context("Unexpeted end of line");

        let path = pop()?.trim_end();

        if path.ends_with('/') {
            continue;
        }
        let (dir, bin) = match path.rfind('/') {
            Some(pos) => path.split_at(pos + 1),
            None => continue,
        };
        if !dir.ends_with("/bin/") {
            continue;
        }

        let _pkgver = pop()?;
        let package_name = strings.add(pop()?);
        let repo = strings.add(pop()?);

        providers.push(Provider {
            repo,
            package_name,
            dir: strings.add(dir),
            bin: strings.add(bin),
        });
    }

    providers.sort_by_key(|provider| strings.get(provider.bin));

    let header = Header {
        version: HEADER_VERSION,

        providers_len: byte_len(&providers),
        strings_len: strings.buf().len().try_into().unwrap(),
    };

    db.write_all(bytes_of(&header))?;
    db.write_all(cast_slice(&providers))?;
    db.write_all(strings.buf())?;

    Ok(())
}
