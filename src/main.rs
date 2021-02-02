mod intern;

use anyhow::{ensure, Context, Result};
use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use fst::MapBuilder;
use getopts::Options;
use intern::{Interner, Span};
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::{env, mem};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Header {
    version: [u8; 16],

    providers_len: u32,
    strings_len: u32,
}

const HEADER_VERSION: [u8; 16] = *b"fcnf version 01\0";

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq)]
#[repr(C)]
struct Provider {
    repo: Span,
    package_name: Span,
    dir: Span,
}

fn index<W: Write>(mut out: W) -> Result<()> {
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

fn search(command: &str) -> Result<()> {
    // let db_file = File::open("./out.db")?;
    // let mmap = unsafe { Mmap::map(&db_file)? };

    // let header: Header = bincode::deserialize(&mmap)?;

    // ensure!(
    //     header.version == HEADER_VERSION,
    //     "unknown header version {:?}",
    //     header.version
    // );

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

fn print_help(opts: Options) {
    print!("{}", opts.usage(env!("CARGO_PKG_NAME")));
}

fn print_version(_opts: Options) {
    println!(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
}

fn main() -> Result<()> {
    let args = env::args().skip(1);

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Print version information");

    let matches = opts.parse(args)?;

    if matches.opt_present("h") {
        print_help(opts);
        return Ok(());
    }

    if matches.opt_present("v") {
        print_version(opts);
        return Ok(());
    }

    let free: Vec<&str> = matches.free.iter().map(AsRef::as_ref).collect();

    match &free[..] {
        ["index"] => index(File::create("./out.db")?)?,
        ["search", command] => search(command)?,
        _ => print_help(opts),
    }

    Ok(())
}
