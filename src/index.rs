use crate::intern::Interner;
use crate::{phf, Header, Provider, Span, HEADER_VERSION};
use anyhow::{Context, Result};
use bytemuck::{bytes_of, cast_slice, Pod};
use std::convert::TryInto;
use std::io::prelude::*;
use std::mem::size_of;

fn bin_providers<'a>(providers: &[Provider], strings: &'a Interner) -> (Vec<&'a str>, Vec<Span>) {
    let mut names: Vec<&str> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();

    for (i, provider) in providers.iter().enumerate() {
        let bin_name = strings.get(provider.bin);
        let duplicate = names
            .last()
            .map(|&name| name == bin_name)
            .unwrap_or(false);

        if duplicate {
            spans.last_mut().unwrap().end += 1;
        } else {
            names.push(bin_name);
            spans.push(Span {
                start: i as u32,
                end: (i + 1) as u32,
            });
        }
    }

    (names, spans)
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

    let (bin_names, bin_spans) = bin_providers(&providers, &strings);

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
        let provider_span = bin_spans[i];

        w.write_all(bytes_of(&provider_span))?;
    }

    w.write_all(strings.buf())?;

    Ok(())
}
