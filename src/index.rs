use crate::intern::Interner;
use crate::{phf, Header, Provider, Span, HEADER_VERSION};
use anyhow::{Context, Result};
use bytemuck::{bytes_of, cast_slice, Pod};
use itertools::Itertools;
use std::convert::TryInto;
use std::io::prelude::*;
use std::mem::size_of;

fn find_providers(providers: &[Provider], strings: &Interner, target: &str) -> Span {
    let start = providers.partition_point(|x| strings.get(x.bin) < target);
    let end = providers[start..]
        .iter()
        .position(|x| strings.get(x.bin) != target)
        .map(|pos| pos + start)
        .unwrap_or(providers.len());

    Span {
        start: start as u32,
        end: end as u32,
    }
}

fn byte_len<T: Pod>(slice: &[T]) -> u32 {
    cast_slice::<T, u8>(slice).len().try_into().unwrap()
}

pub fn index(r: impl BufRead, mut w: impl Write) -> Result<()> {
    let mut strings = Interner::new();
    let mut providers = Vec::new();

    for line in r.lines() {
        let line = line.context("pacman stdout not valid UTF-8")?;

        let mut parts = line.rsplit('\0');
        let mut pop = || parts.next().context("Unexpeted end of line");

        let path = pop()?;

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

    providers.sort_unstable_by_key(|provider| strings.get(provider.bin));
    let bin_names: Vec<&str> = providers
        .iter()
        .map(|provider| strings.get(provider.bin))
        .dedup()
        .collect();

    let hash_state = phf::generate_hash(&bin_names);
    assert_eq!(bin_names.len(), hash_state.map.len());

    let header = Header {
        version: HEADER_VERSION,

        hash_key: hash_state.key,

        providers_len: byte_len(&providers),
        disps_len: byte_len(&hash_state.disps),
        table_len: (bin_names.len() * size_of::<Span>()).try_into().unwrap(),
        strings_len: strings.buf().len().try_into().unwrap(),
    };

    w.write_all(bytes_of(&header))?;
    w.write_all(cast_slice(&providers))?;
    w.write_all(cast_slice(&hash_state.disps))?;

    for &i in &hash_state.map {
        let bin = bin_names[i];
        let provider_span = find_providers(&providers, &strings, bin);

        w.write_all(bytes_of(&provider_span))?;
    }

    w.write_all(strings.buf())?;

    Ok(())
}
