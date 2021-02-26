use crate::intern::{Interner, Span};
use crate::phf;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Context, Result};
use bytemuck::{bytes_of, cast_slice, Pod};
use itertools::Itertools;
use std::cmp::Ordering::{Greater, Less};
use std::convert::TryInto;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem::size_of;
use std::process::{Command, Stdio};

fn find_providers(providers: &[Provider], buf: &[u8], target: &str) -> Span {
    fn partition_point(slice: &[Provider], pred: impl Fn(Provider) -> bool) -> usize {
        slice
            .binary_search_by(|&x| if pred(x) { Less } else { Greater })
            .unwrap_err()
    }

    let start = partition_point(providers, |x| x.bin.get(buf) < target);
    let end = providers[start..]
        .iter()
        .position(|x| x.bin.get(buf) != target)
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

pub fn index<W: Write>(mut out: W) -> Result<()> {
    let mut child = Command::new("cat")
        .args(&["list"])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());

    let mut strings = Interner::new();
    let mut providers = Vec::new();

    for line in stdout.lines() {
        let line = line?;

        let mut parts = line.rsplit('\0');
        let mut pop = || parts.next().context("unexpeted end of line");

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

        let bin = strings.add(bin);
        let dir = strings.add(dir);

        providers.push(Provider {
            repo: repo.span,
            package_name: package_name.span,
            dir: dir.span,
            bin: bin.span,
        });
    }

    let status = child.wait()?;
    ensure!(status.success(), "pacman failed: {}", status);

    let buf = strings.buf();

    providers.sort_unstable_by_key(|provider| provider.bin.get(buf));
    let bin_names: Vec<&str> = providers
        .iter()
        .map(|provider| provider.bin.get(buf))
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
        strings_len: buf.len().try_into().unwrap(),
    };

    out.write_all(bytes_of(&header))?;
    out.write_all(cast_slice(&providers))?;
    out.write_all(cast_slice(&hash_state.disps))?;

    for &i in &hash_state.map {
        let bin = bin_names[i];
        let provider_span = find_providers(&providers, buf, bin);

        out.write_all(bytes_of(&provider_span))?;
    }

    out.write_all(buf)?;

    Ok(())
}
