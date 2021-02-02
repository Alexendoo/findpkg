mod intern;

use alpm::{Alpm, SigLevel, Usage};
use anyhow::{ensure, Context, Result};
use flate2::read::GzDecoder;
use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, MapBuilder, Streamer};
use getopts::Options;
use memmap::Mmap;
use once_cell::unsync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, HashMap}, io::BufReader, process::Stdio};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use intern::Interner;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::{Archive, EntryType};

#[derive(Serialize, Deserialize, Debug)]
struct Header {
    version: [u8; 16],
    fst_len: u64,
}

const HEADER_VERSION: [u8; 16] = *b"fcnf version 01\0";

#[derive(Serialize, Deserialize)]
struct Provider {
    package_name: u32,
    path: u32,
}

fn index() -> Result<()> {
    let mut child = Command::new("pacman")
        .args(&["-Fl", "--machinereadable"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());

    let mut paths = Interner::new();
    let mut repos = Interner::new();

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
        let package_name = pop()?;
        let repo = pop()?;

        println!("{}", dir);

        // bins_owned.entry(bin.to_string()).push(Provider {
        //     package_name,
        //     path,
        // });
    }

    let status = child.wait()?;
    ensure!(status.success(), "pacman failed: {}", status);

    // println!("{:#?}", bins);

    //    let packages = read_packages(core)?;
    //    let mut bins = BTreeMap::new();
    //    put_providers(&packages, &mut bins)?;
    //
    //    let mut builder = MapBuilder::memory();
    //    let mut buffer: Vec<u8> = Vec::new();
    //
    //    for (bin, providers) in bins {
    //        let index = buffer.len() as u64;
    //        bincode::serialize_into(&mut buffer, &providers)?;
    //
    //        builder.insert(bin, index)?;
    //    }
    //
    //    let map = builder.into_inner()?;
    //
    //    let header = Header {
    //        version: HEADER_VERSION,
    //        fst_len: map.len() as u64,
    //    };
    //
    //    let mut out = File::create("./out.db")?;
    //
    //    bincode::serialize_into(&mut out, &header)?;
    //    out.write_all(&map)?;
    //    out.write_all(&buffer)?;

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
        ["index"] => index()?,
        ["search", command] => search(command)?,
        _ => print_help(opts),
    }

    Ok(())
}
