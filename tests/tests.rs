use anyhow::Result;
use findpkg::search::Database;
use findpkg::update::index;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::{env, fmt, fs};
use unindent::unindent;

macro_rules! include_db {
    ($file:literal) => {{
        #[repr(C, align(4096))]
        struct PageAligned<T: ?Sized>(T);
        static ALIGNED: &PageAligned<[u8]> = &PageAligned(*include_bytes!($file));

        &ALIGNED.0
    }};
}

#[track_caller]
fn assert_str_eq(left: &str, right: &str) {
    #[derive(PartialEq)]
    struct DisplayAsDebug<'a>(&'a str);

    impl fmt::Debug for DisplayAsDebug<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0.replace('\t', "\\t"))
        }
    }

    assert_eq!(DisplayAsDebug(left), DisplayAsDebug(right));
}

static DB_BYTES: &[u8] = include_db!("database");

#[test]
fn create_small() -> Result<()> {
    let list = include_str!("list.csv").replace(',', "\0");

    let mut db = Vec::new();

    index(list.as_bytes(), &mut db)?;

    if db != DB_BYTES {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("database");
        fs::write(&path, db).unwrap();
        panic!("Database did not match, generated: {}", path.display());
    }

    Ok(())
}

#[test]
fn found() -> Result<()> {
    let cases = &[
        (
            "openssl",
            unindent("
                openssl may be found in the following packages:
                  core/openssl\t/usr/bin/openssl
            "),
        ),
        (
            "ecryptfs-insert-wrapped-passphrase-into-keyring",
            unindent("
                ecryptfs-insert-wrapped-passphrase-into-keyring may be found in the following packages:
                  community/ecryptfs-utils\t/usr/bin/ecryptfs-insert-wrapped-passphrase-into-keyring
            "),
        ),
        (
            "R",
            unindent("
                R may be found in the following packages:
                  extra/r\t/usr/bin/R
                  extra/r\t/usr/lib/R/bin/R
            "),
        ),
        (
            "ld",
            unindent("
                ld may be found in the following packages:
                  core/binutils                       \t/usr/bin/ld
                  community/aarch64-linux-gnu-binutils\t/usr/aarch64-linux-gnu/bin/ld
                  community/arm-none-eabi-binutils    \t/usr/arm-none-eabi/bin/ld
                  community/lm32-elf-binutils         \t/usr/lm32-elf/bin/ld
                  community/mingw-w64-binutils        \t/usr/i686-w64-mingw32/bin/ld
                  community/mingw-w64-binutils        \t/usr/x86_64-w64-mingw32/bin/ld
                  community/nds32le-elf-binutils      \t/usr/nds32le-elf/bin/ld
                  community/or1k-elf-binutils         \t/usr/or1k-elf/bin/ld
                  community/ppc64le-elf-binutils      \t/usr/ppc64le-elf/bin/ld
                  community/riscv32-elf-binutils      \t/usr/riscv32-elf/bin/ld
                  community/riscv64-elf-binutils      \t/usr/riscv64-elf/bin/ld
                  community/riscv64-linux-gnu-binutils\t/usr/riscv64-linux-gnu/bin/ld
                  community/sh2-elf-binutils          \t/usr/sh2-elf/bin/ld
                  community/sh4-elf-binutils          \t/usr/sh4-elf/bin/ld
            "),
        ),
        (
            "zzxordir",
            unindent("
                zzxordir may be found in the following packages:
                  extra/zziplib\t/usr/bin/zzxordir
            "),
        ),
    ];

    let db = Database::new(DB_BYTES)?;

    for (command, expected) in cases {
        assert_str_eq(expected, &db.search(command).unwrap());
    }

    Ok(())
}

#[test]
fn not_found() -> Result<()> {
    let cases = &[
        "LS",
        "",
        "\0",
        " ",
        "\n",
        "nxcommand",
        "vendor_perl",
        "__pycache__",
        #[cfg(not(miri))]
        &"a-long-name-".repeat(8000),
    ];

    let db = Database::new(DB_BYTES)?;

    for &command in cases {
        if let Some(msg) = db.search(command) {
            panic!("Found {}: {}", command, msg);
        }
    }

    Ok(())
}
