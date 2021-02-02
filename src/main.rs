mod index;
mod intern;
mod search;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use getopts::Options;
use index::index;
use intern::Span;
use search::search;
use std::env;
use std::fs::File;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Header {
    version: [u8; 16],

    providers_len: u32,
    strings_len: u32,
}

pub const HEADER_VERSION: [u8; 16] = *b"fcnf version 01\0";

#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq, Eq)]
#[repr(C)]
pub struct Provider {
    repo: Span,
    package_name: Span,
    dir: Span,
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
