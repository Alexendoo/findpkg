use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Result};
use bytemuck::{cast_slice, from_bytes, Pod};
use std::fmt::{self, Write};
use std::{mem, str};

fn split_cast<T: Pod>(slice: &[u8], mid: u32) -> (&[T], &[u8]) {
    let (bytes, rest) = slice.split_at(mid as usize);
    (cast_slice(bytes), rest)
}

pub enum Entry {
    Found(String),
    NotFound,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Database<'a> {
    header: Header,
    providers: &'a [Provider],
    strings: &'a [u8],
}

impl<'a> Database<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        let (header_bytes, rest) = bytes.split_at(mem::size_of::<Header>());
        let header: Header = *from_bytes(header_bytes);

        ensure!(
            header.version == HEADER_VERSION,
            "unknown header version {:?}",
            String::from_utf8_lossy(&header.version),
        );

        let (providers, strings) = split_cast::<Provider>(rest, header.providers_len);

        Ok(Self {
            header,
            providers,
            strings,
        })
    }

    pub fn search(self, command: &str) -> Option<String> {
        let start = self
            .providers
            .partition_point(|provider| self.get(provider.bin) < command);
        let end = self.providers[start..]
            .iter()
            .position(|provider| self.get(provider.bin) != command)
            .map(|length| start + length)
            .unwrap_or(self.providers.len());

        let matches = &self.providers[start..end];

        if matches.is_empty() {
            return None;
        }

        let max_len = matches
            .iter()
            .map(|prov| self.get(prov.package).len())
            .max()
            .unwrap();

        let mut out = format!("{} may be found in the following packages:\n", command);

        for provider in matches {
            writeln!(
                out,
                "  {:padding$}\t/{}{}",
                self.get(provider.package),
                self.get(provider.dir),
                self.get(provider.bin),
                padding = max_len,
            )
            .unwrap();
        }

        Some(out)
    }

    fn get(&self, offset: u32) -> &str {
        let s = &self.strings[offset as usize..];
        let end = s
            .iter()
            .position(|&b| b == b'\n')
            .expect("Unterminated string");

        str::from_utf8(&s[..end]).unwrap()
    }
}

impl<'a> fmt::Debug for Database<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for provider in self.providers {
            writeln!(
                f,
                "{}\t{}{}",
                self.get(provider.package),
                self.get(provider.dir),
                self.get(provider.bin)
            )?;
        }

        Ok(())
    }
}
