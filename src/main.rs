use anyhow::{anyhow, Result};
use findpkg::search::Database;
use findpkg::update::{update_pacman, update_stdin};
use getopts::Options;
use memmap::Mmap;
use std::fs::File;
use std::io::ErrorKind;
use std::process::exit;
use std::{env, str};

fn print_help(opts: Options) {
    const USAGE: &str = "Usage:
    findpkg [OPTIONS] COMMAND
        Shows any known packages that provide COMMAND

        e.g. `findpkg units` would display:

        units may be found in the following packages:
          community/units    \t/usr/bin/units
          community/plan9port\t/usr/lib/plan9/bin/units";

    print!("{}", opts.usage(USAGE));
}

fn print_version(_opts: Options) {
    println!(concat!("findpkg v", env!("CARGO_PKG_VERSION")));
}

fn main() -> Result<()> {
    let args = env::args().skip(1);

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("v", "version", "Print version information");
    opts.optopt(
        "f",
        "database",
        "Location of the database (default: /var/lib/findpkg/database)",
        "FILE",
    );
    opts.optflag("u", "update", "Update the database");
    opts.optflag("", "offline", "Don't run pacman -Fy");
    opts.optflag("", "stdin", "Read from stdin rather than pacman");

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
    let db_path = db_path.as_deref().unwrap_or("/var/lib/findpkg/database");

    if matches.opt_present("stdin") {
        return update_stdin(db_path);
    }

    if matches.opt_present("update") {
        return update_pacman(db_path, matches.opt_present("offline"));
    }

    if matches.free.len() != 1 {
        print_help(opts);
        exit(1);
    }

    let command = &matches.free[0];

    let db_file = File::open(db_path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => anyhow!(
            "Database file not found: {}\n\nTry running `findpkg --update`",
            db_path
        ),
        _ => anyhow!("Failed to open database {}\n\n{}", db_path, e),
    })?;
    let mmap = unsafe { Mmap::map(&db_file)? };

    match Database::new(&mmap)?.search(command) {
        Some(msg) => print!("{}", msg),
        None => println!("Command not found: {}", command),
    }

    Ok(())
}
