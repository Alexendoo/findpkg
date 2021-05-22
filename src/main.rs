mod index;
mod intern;
mod phf;
mod search;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use getopts::Options;
use index::index;
use search::search;
use std::{env, str};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Header {
    version: [u8; 16],

    hash_key: u64,

    providers_len: u32,
    disps_len: u32,
    table_len: u32,
    strings_len: u32,
}

pub const HEADER_VERSION: [u8; 16] = *b"fcnf format 001\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn get<T>(self, slice: &[T]) -> &[T] {
        &slice[self.start as usize..self.end as usize]
    }

    pub fn get_str(self, bytes: &[u8]) -> &str {
        str::from_utf8(self.get(bytes)).unwrap()
    }

    pub fn len(self) -> usize {
        (self.end - self.start) as usize
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq)]
#[repr(C)]
pub struct Provider {
    repo: Span,
    package_name: Span,
    dir: Span,
    bin: Span,
}

fn print_help(opts: Options) {
    const USAGE: &str = "Usage:
    fast-command-not-found [OPTIONS] COMMAND
        Shows any known packages that provide COMMAND

        e.g. `fast-command-not-found units` would display:

        community/units    \t/usr/bin/units
        community/plan9port\t/usr/lib/plan9/bin/units";

    print!("{}", opts.usage(USAGE));
}

fn print_version(_opts: Options) {
    println!(concat!(
        "fast-command-not-found v",
        env!("CARGO_PKG_VERSION")
    ));
}

fn main() -> Result<()> {
    let args = env::args().skip(1);

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Print version information");
    opts.optopt(
        "",
        "database",
        "Location of the database (default: /var/lib/fast-command-not-found/database)",
        "FILE",
    );
    opts.optflag("u", "update", "Update the database");

    let matches = opts.parse(args)?;

    if matches.opt_present("help") {
        print_help(opts);
        return Ok(());
    }

    if matches.opt_present("version") {
        print_version(opts);
        return Ok(());
    }

    let db_path = matches.opt_str("database");
    let db_path = db_path
        .as_deref()
        .unwrap_or("/var/lib/fast-command-not-found/database");

    if matches.opt_present("update") {
        index(db_path)?;
        return Ok(());
    }
    
    if let [command] = &*matches.free {
        if !search(command, db_path)? {
            std::process::exit(127);
        }
    } else {
        print_help(opts);
    }

    Ok(())
}
