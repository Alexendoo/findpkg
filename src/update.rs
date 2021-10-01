use crate::intern::Interner;
use crate::{Header, Provider, HEADER_VERSION};
use anyhow::{ensure, Context, Result};
use bstr::io::BufReadExt;
use bstr::ByteSlice;
use bytemuck::{bytes_of, cast_slice, Pod};
use std::convert::TryInto;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

fn byte_len<T: Pod>(slice: &[T]) -> u32 {
    cast_slice::<T, u8>(slice).len().try_into().unwrap()
}

pub fn index(list: impl BufRead, mut db: impl Write) -> Result<()> {
    let mut strings = Interner::new();
    let mut providers = Vec::new();

    list.for_byte_line(|line| {
        let mut parts = line.rsplit(|&ch| ch == b'\0');
        let mut pop = || parts.next().expect("Unexpeted end of line").as_bstr();

        let path = pop();

        if path.ends_with(b"/") {
            return Ok(true);
        }
        let (dir, bin) = match path.rfind_byte(b'/') {
            Some(pos) => path.split_at(pos + 1),
            None => return Ok(true),
        };
        if !dir.ends_with(b"/bin/") {
            return Ok(true);
        }

        let _pkgver = pop();
        let package_name = pop();
        let repo = pop();

        providers.push(Provider {
            package: strings.add(format!("{}/{}", repo, package_name).as_bytes()),
            dir: strings.add(dir),
            bin: strings.add(bin),
        });

        Ok(true)
    })?;

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

fn update(db_path: &str, reader: impl BufRead) -> Result<()> {
    let db_dir = Path::new(db_path)
        .parent()
        .with_context(|| format!("Invalid DB path: {}", db_path))?;
    fs::create_dir_all(db_dir)?;

    let temp_path = format!("{}.tmp", db_path);
    let mut out = File::create(&temp_path)
        .with_context(|| format!("Failed to create file: {}", &temp_path))?;

    index(reader, &mut out)?;

    out.sync_all()?;
    drop(out);

    fs::rename(&temp_path, db_path)
        .with_context(|| format!("Failed to rename {} -> {}", &temp_path, db_path))?;

    Ok(())
}

pub fn update_pacman(db_path: &str, offline: bool) -> Result<()> {
    if !offline {
        let status = Command::new("pacman")
            .arg("-Fy")
            .status()
            .context("Failed to run pacman")?;
        ensure!(status.success(), "Pacman failed: {}", status);
    }

    let mut child = Command::new("pacman")
        .args(&["-Fl", "--machinereadable"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run pacman")?;

    let child_stdout = BufReader::new(child.stdout.take().unwrap());

    update(db_path, child_stdout)?;

    let status = child.wait()?;
    ensure!(status.success(), "Pacman failed: {}", status);

    Ok(())
}

pub fn update_stdin(db_path: &str) -> Result<()> {
    let stdin = io::stdin();
    update(db_path, stdin.lock())
}
