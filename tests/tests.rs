use anyhow::Result;
use fast_command_not_found::index::index;
use fast_command_not_found::search::{search, Entry};
use indoc::indoc;
use pretty_assertions::assert_eq;
use std::io::BufReader;
use zstd::Decoder;

static DB: &[u8] = {
    #[repr(C, align(4096))]
    struct PageAligned<T: ?Sized>(T);
    static ALIGNED: &PageAligned<[u8]> = &PageAligned(*include_bytes!("database"));

    &ALIGNED.0
};
static LIST_ZST: &[u8] = include_bytes!("list.zst");

#[test]
fn create() -> Result<()> {
    let reader = BufReader::new(Decoder::new(LIST_ZST)?);
    let mut db = Vec::new();

    index(reader, &mut db)?;

    assert!(db == DB);

    Ok(())
}

#[test]
fn found() -> Result<()> {
    let cases = [
        (
            "ls",
            indoc! {"
                community/9base    \t/opt/plan9/bin/ls
                core/coreutils     \t/usr/bin/ls
                community/plan9port\t/usr/lib/plan9/bin/ls
            "},
        ),
        (
            "openssl",
            indoc! {"
                core/openssl\t/usr/bin/openssl
            "},
        ),
        (
            "ecryptfs-insert-wrapped-passphrase-into-keyring",
            indoc! {"
                community/ecryptfs-utils\t/usr/bin/ecryptfs-insert-wrapped-passphrase-into-keyring
            "},
        ),
        (
            "R",
            indoc! {"
                extra/r\t/usr/lib/R/bin/R
                extra/r\t/usr/bin/R
            "},
        ),
        (
            "ld",
            indoc! {"
                community/lm32-elf-binutils         \t/usr/lm32-elf/bin/ld
                community/riscv32-elf-binutils      \t/usr/riscv32-elf/bin/ld
                community/nds32le-elf-binutils      \t/usr/nds32le-elf/bin/ld
                community/mingw-w64-binutils        \t/usr/i686-w64-mingw32/bin/ld
                community/aarch64-linux-gnu-binutils\t/usr/aarch64-linux-gnu/bin/ld
                community/ppc64le-elf-binutils      \t/usr/ppc64le-elf/bin/ld
                community/arm-none-eabi-binutils    \t/usr/arm-none-eabi/bin/ld
                community/sh2-elf-binutils          \t/usr/sh2-elf/bin/ld
                community/sh4-elf-binutils          \t/usr/sh4-elf/bin/ld
                community/riscv64-elf-binutils      \t/usr/riscv64-elf/bin/ld
                community/or1k-elf-binutils         \t/usr/or1k-elf/bin/ld
                community/mingw-w64-binutils        \t/usr/x86_64-w64-mingw32/bin/ld
                community/riscv64-linux-gnu-binutils\t/usr/riscv64-linux-gnu/bin/ld
                core/binutils                       \t/usr/bin/ld
            "},
        ),
    ];

    for (command, expected) in cases {
        match search(command, DB)? {
            Entry::Found(msg) => {
                assert_eq!(msg, expected)
            }
            Entry::NotFound => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn not_found() -> Result<()> {
    let long = "a-long-name-".repeat(8000);
    let cases = [
        "LS",
        "",
        "\0",
        " ",
        "\n",
        "nxcommand",
        &long,
        "vendor_perl",
        "__pycache__",
    ];

    for command in cases {
        match search(command, DB)? {
            Entry::Found(_) => unreachable!(),
            Entry::NotFound => {}
        }
    }

    Ok(())
}
