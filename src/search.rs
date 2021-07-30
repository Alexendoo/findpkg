use crate::{phf, Header, Provider, Span, HEADER_VERSION};
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
    disps: &'a [phf::Disp],
    table: &'a [Span],
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

        let (providers, rest) = split_cast::<Provider>(rest, header.providers_len);
        let (disps, rest) = split_cast::<phf::Disp>(rest, header.disps_len);
        let (table, strings) = split_cast::<Span>(rest, header.table_len);

        Ok(Self {
            header,
            providers,
            disps,
            table,
            strings,
        })
    }

    pub fn search(self, command: &str) -> Result<Entry> {
        let hashes = phf::hash(command, self.header.hash_key);
        let index = phf::get_index(&hashes, self.disps, self.table.len());

        let providers_span = self.table[index as usize];
        let bin_providers = providers_span.get(self.providers);

        if bin_providers[0].bin.get(self.strings) != command.as_bytes() {
            return Ok(Entry::NotFound);
        }

        let max_len = bin_providers
            .iter()
            .map(|prov| prov.repo.len() + prov.package_name.len())
            .max()
            .unwrap();

        let mut out = String::new();

        for provider in bin_providers {
            let repo = self.get_str(provider.repo);

            writeln!(
                out,
                "{}/{:padding$}\t/{}{}",
                repo,
                self.get_str(provider.package_name),
                self.get_str(provider.dir),
                self.get_str(provider.bin),
                padding = max_len - repo.len(),
            )?;
        }

        Ok(Entry::Found(out))
    }

    fn get_str(&self, span: Span) -> &str {
        span.get_str(self.strings)
    }
}

impl<'a> fmt::Debug for Database<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for providers_span in self.table {
            let bin_providers = providers_span.get(self.providers);

            for provider in bin_providers {
                writeln!(
                    f,
                    "{}/{}\t{}{}",
                    self.get_str(provider.repo),
                    self.get_str(provider.package_name),
                    self.get_str(provider.dir),
                    self.get_str(provider.bin)
                )?;
            }
        }

        Ok(())
    }
}
