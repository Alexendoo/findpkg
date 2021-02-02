use crate::intern::Interner;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Context, Result};
use bytemuck::{bytes_of, cast_slice};
use fst::MapBuilder;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::mem;
use std::process::{Command, Stdio};

pub fn index<W: Write>(mut out: W) -> Result<()> {
    let mut child = Command::new("cat")
        .args(&["list"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());

    let mut strings = Interner::new();
    let mut bins = BTreeMap::<&str, Vec<Provider>>::new();
    let mut providers_len = 0;

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

        providers_len += mem::size_of::<Provider>() as u32;
        bins.entry(bin.str).or_default().push(Provider {
            repo: repo.span,
            package_name: package_name.span,
            dir: dir.span,
        });
    }

    let status = child.wait()?;
    ensure!(status.success(), "pacman failed: {}", status);

    let header = Header {
        version: HEADER_VERSION,

        providers_len,
        strings_len: strings.buf().len() as u32,
    };

    out.write_all(bytes_of(&header))?;

    for (_, providers) in &bins {
        out.write_all(cast_slice(&providers))?;
    }

    out.write_all(strings.buf().as_bytes())?;

    let mut builder = MapBuilder::new(&mut out)?;

    let mut provider_offset = 0;
    for (bin, providers) in bins {
        let len_shifted = (providers.len() as u64) << 32;
        builder.insert(bin, provider_offset | len_shifted)?;

        provider_offset += providers.len() as u64;
    }

    Ok(())
}
