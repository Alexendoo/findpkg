use anyhow::Result;
use fast_command_not_found::search::Database;
use fast_command_not_found::update::index;
use pretty_assertions::assert_eq;
use std::fmt;
use std::io::BufReader;
use unindent::unindent;
use zstd::Decoder;

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

static DB: &[u8] = include_db!("database");
static SMALL_DB: &[u8] = include_db!("small-database");
static LIST_ZST: &[u8] = include_bytes!("list.zst");

#[test]
#[cfg_attr(miri, ignore)]
fn create_full() -> Result<()> {
    let list = BufReader::new(Decoder::new(LIST_ZST)?);
    let mut db = Vec::new();

    index(list, &mut db)?;

    assert_eq!(Database::new(DB)?, Database::new(&db)?);
    assert!(db == DB);

    Ok(())
}

#[test]
fn create_small() -> Result<()> {
    #[rustfmt::skip]
    let list = unindent("
        core\0dash\00.5.11.3-1\0usr/
        core\0dash\00.5.11.3-1\0usr/bin/
        core\0dash\00.5.11.3-1\0usr/bin/dash
        core\0dash\00.5.11.3-1\0usr/share/
        core\0dash\00.5.11.3-1\0usr/share/licenses/
        core\0dash\00.5.11.3-1\0usr/share/licenses/dash/
        core\0dash\00.5.11.3-1\0usr/share/licenses/dash/COPYING
        core\0dash\00.5.11.3-1\0usr/share/man/
        core\0dash\00.5.11.3-1\0usr/share/man/man1/
        core\0dash\00.5.11.3-1\0usr/share/man/man1/dash.1.gz
        core\0diffutils\03.7-3\0usr/
        core\0diffutils\03.7-3\0usr/bin/
        core\0diffutils\03.7-3\0usr/bin/cmp
        core\0diffutils\03.7-3\0usr/bin/diff
        core\0diffutils\03.7-3\0usr/bin/diff3
        core\0diffutils\03.7-3\0usr/bin/sdiff
        core\0diffutils\03.7-3\0usr/share/
        core\0diffutils\03.7-3\0usr/share/info/
        core\0diffutils\03.7-3\0usr/share/info/diffutils.info.gz
        core\0dnssec-anchors\020190629-3\0etc/
        core\0dnssec-anchors\020190629-3\0etc/trusted-key.key
        core\0dnssec-anchors\020190629-3\0usr/
        core\0dnssec-anchors\020190629-3\0usr/share/
        core\0dnssec-anchors\020190629-3\0usr/share/licenses/
        core\0dnssec-anchors\020190629-3\0usr/share/licenses/dnssec-anchors/
        core\0dnssec-anchors\020190629-3\0usr/share/licenses/dnssec-anchors/LICENSE
        extra\0tree\01.8.0-2\0usr/
        extra\0tree\01.8.0-2\0usr/bin/
        extra\0tree\01.8.0-2\0usr/bin/tree
        extra\0tree\01.8.0-2\0usr/share/
        extra\0tree\01.8.0-2\0usr/share/man/
        extra\0tree\01.8.0-2\0usr/share/man/man1/
        extra\0tree\01.8.0-2\0usr/share/man/man1/tree.1.gz
        community\0weechat\03.0-2\0usr/
        community\0weechat\03.0-2\0usr/bin/
        community\0weechat\03.0-2\0usr/bin/weechat
        community\0weechat\03.0-2\0usr/bin/weechat-curses
        community\0weechat\03.0-2\0usr/bin/weechat-headless
    ");

    let mut db = Vec::new();

    index(list.as_bytes(), &mut db)?;

    assert!(db == SMALL_DB);

    Ok(())
}

#[test]
fn found() -> Result<()> {
    let cases = &[
        (
            "ls",
            unindent("
                ls may be found in the following packages:
                  core/coreutils     \t/usr/bin/ls
                  community/9base    \t/opt/plan9/bin/ls
                  community/plan9port\t/usr/lib/plan9/bin/ls
            "),
        ),
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

    let db = Database::new(DB)?;

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

    let db = Database::new(DB)?;

    for &command in cases {
        if let Some(msg) = db.search(command) {
            panic!("Found {}: {}", command, msg);
        }
    }

    Ok(())
}
