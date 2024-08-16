use flate2::read::MultiGzDecoder;
use memmap2::Mmap;
use rayon::prelude::*;
use std::error::Error;
use std::{
    fmt::Debug,
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

pub fn get_sizes<T: AsRef<Path> + Debug>(fasta: T) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let path = fasta.as_ref();
    let ext = path.extension().unwrap();
    let file = File::open(path)?;

    let lines = match ext.to_str().unwrap() {
        "gz" => with_gz(&file)?,
        "fa" | "fasta" | "fna" => raw(&file)?,
        _ => panic!("ERROR: Not a fasta. Wrong file format!"),
    };

    Ok(lines)
}

pub fn raw(file: &File) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let mmap = unsafe { Mmap::map(file)? };
    let lines = chromsize(&mmap)?;

    Ok(lines)
}

fn with_gz(file: &File) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let mmap = unsafe { Mmap::map(file)? };
    let mut decoder = MultiGzDecoder::new(&mmap[..]);

    let mut buffer = Vec::with_capacity(100 * 1024 * 1024); // 100MB buffer
    decoder.read_to_end(&mut buffer)?;

    let lines = chromsize(&buffer)?;

    Ok(lines)
}

fn chromsize(data: &[u8]) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let lines = data
        .par_split(|&c| c == b'>')
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            let mut totals = 0u64;
            let stop = memchr::memchr(b'\n', chunk).unwrap_or(0);
            let chr = unsafe {
                std::str::from_utf8_unchecked(&chunk[..stop])
                    .trim()
                    .to_string()
            };
            let data = &chunk[stop + 1..];
            for line in data.split(|&c| c == b'\n') {
                totals += unsafe { std::str::from_utf8_unchecked(line).trim().len() as u64 };
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
