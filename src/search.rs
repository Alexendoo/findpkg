use anyhow::Result;

pub fn search(command: &str) -> Result<()> {
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
