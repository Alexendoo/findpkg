use anyhow::{ensure, Context, Result};
use once_cell::unsync::Lazy;
use regex::{Match, Regex};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use tar::{Archive, EntryType};
use tempfile::TempDir;

#[derive(Debug, Default)]
struct Package {
    desc: String,
    files: String,
}

fn re(section: &str) -> Regex {
    Regex::new(&format!(r"%{}%\n([^%]+)\n", section)).unwrap()
}
const NAME_RE: Lazy<Regex> = Lazy::new(|| re("NAME"));
const FILES_RE: Lazy<Regex> = Lazy::new(|| re("FILES"));

fn get_section<'a>(re: &Regex, text: &'a str) -> Option<Match<'a>> {
    re.captures(text)?.get(1)
}

fn tempdir() -> io::Result<TempDir> {
    tempfile::Builder::new()
        .prefix(env!("CARGO_PKG_NAME"))
        .tempdir()
}

fn read_packages(reader: impl Read) -> Result<HashMap<PathBuf, Package>> {
    let mut archive = Archive::new(reader);
    let mut packages = HashMap::<PathBuf, Package>::new();

    for file in archive.entries()? {
        let mut file = file?;

        if file.header().entry_type() != EntryType::Regular {
            continue;
        }

        let path = file.path()?;
        let parent = path.parent().context("invalid parent")?.to_owned();
        let file_name = path.file_name().context("invalid filename")?;

        let package = packages.entry(parent).or_default();
        let write = match file_name.to_str() {
            Some("desc") => &mut package.desc,
            Some("files") => &mut package.files,
            _ => continue,
        };
        file.read_to_string(write)?;
    }

    for (path, package) in &packages {
        ensure!(!package.desc.is_empty(), "expected package.desc: {:?}", path);
        ensure!(!package.files.is_empty(), "expected package.files: {:?}", path);
    }

    Ok(packages)
}

fn main() -> Result<()> {
    let core = File::open("./core.files")?;

    let packages = read_packages(core)?;

    Ok(())
}
