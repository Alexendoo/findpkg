use anyhow::{anyhow, ensure, Context, Result};
use fast_command_not_found::index::index;
use fast_command_not_found::search::{Database, Entry};
use getopts::Options;
use memmap::Mmap;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind};
use std::process::{Command, Stdio};
use std::{env, str};

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
        let temp_path = format!("{}.tmp", db_path);
        let mut out = File::create(&temp_path)
            .with_context(|| format!("Failed to create file: {}", &temp_path))?;

        let mut child = Command::new("pacman")
            .args(&["-Fl", "--machinereadable"])
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to run pacman")?;

        let child_stdout = BufReader::new(child.stdout.take().unwrap());

        index(child_stdout, &mut out)?;

        let status = child.wait()?;
        ensure!(status.success(), "Pacman failed: {}", status);

        out.sync_all()?;
        drop(out);

        fs::rename(&temp_path, db_path)
            .with_context(|| format!("Failed to rename {} -> {}", &temp_path, db_path))?;

        return Ok(());
    }

    if let [command] = &*matches.free {
        let db_file = File::open(db_path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => anyhow!(
                "Database file not found: {}\n\nTry running `fast-command-not-found --update`",
                db_path
            ),
            _ => anyhow!("Failed to open database {}\n\n{}", db_path, e),
        })?;
        let mmap = unsafe { Mmap::map(&db_file)? };

        match Database::new(&mmap)?.search(command)? {
            Entry::Found(msg) => print!("{}", msg),
            Entry::NotFound => println!("Command not found: {}", command),
        }
    } else {
        print_help(opts);
    }

    Ok(())
}
