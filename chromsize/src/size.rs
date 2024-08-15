use memchr::memchr;
use memmap2::Mmap;
use rayon::prelude::*;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    str::from_utf8_unchecked,
};

pub fn chromsize<T: AsRef<Path> + Debug>(
    fasta: T,
) -> Result<Vec<(String, u64)>, Box<dyn std::error::Error>> {
    let file = File::open(fasta)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let lines = mmap
        .par_split(|&c| c == b'>')
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            let mut totals = 0u64;
            let stop = memchr(b'\n', chunk).unwrap_or(0);
            let chr = unsafe { from_utf8_unchecked(&chunk[..stop]).trim().to_string() };

            let data = &chunk[stop + 1..];
            for line in data.split(|&c| c == b'\n') {
                totals += unsafe { from_utf8_unchecked(line).trim().len() as u64 };
            }

            (chr, totals)
        })
        .collect::<Vec<(String, u64)>>();

    Ok(lines)
}

pub fn writer<T>(sizes: Vec<(String, u64)>, out: T)
where
    T: AsRef<Path> + Debug,
{
    let o = match File::create(out) {
        Ok(f) => f,
        Err(e) => panic!("Error creating file: {}", e),
    };
    let mut writer = BufWriter::new(o);

    for (k, v) in sizes.iter() {
        writeln!(writer, "{}\t{}", k, v).unwrap();
    }
}
