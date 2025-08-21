//! chromsize
//! Alejandro Gonzales-Irribarren, 2024
//!
//! `chromsize` is a utility designed to extract chromosome names
//! and their corresponding lengths from FASTA files. It supports
//! both plain and gzipped FASTA formats and offers an option to
//! include only the accession ID from the FASTA headers.

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

/// Retrieves the sizes (chromosome name and length) from a FASTA or gzipped FASTA file.
///
/// This function determines the file type (plain or gzipped FASTA) based on its
/// extension and then calls the appropriate parsing function (`raw` or `with_gz`)
/// to extract chromosome names and their total sequence lengths.
///
/// # Type Parameters
/// * `T` - The type of the file path, which must implement `AsRef<Path>` and `Debug`.
///
/// # Arguments
/// * `fasta` - The path to the input FASTA or gzipped FASTA file.
/// * `accession_only` - If `true`, only the accession part of the FASTA header
///                      (before the first space) will be used as the chromosome name.
///                      Otherwise, the entire header line up to the first newline will be used.
///
/// # Returns
/// A `Result` containing a `Vec` of `(String, u64)` tuples, where each tuple
/// represents `(chromosome_name, chromosome_length)`, or a `Box<dyn Error>` if
/// an I/O or parsing error occurs, or if the file format is not recognized.
///
/// # Panics
/// * If the file extension cannot be determined or is not one of "gz", "fa", "fasta", or "fna".
/// * If the underlying `raw` or `with_gz` functions panic due to file reading issues.
///
/// # Example
/// ```rust, ignore
/// use std::path::PathBuf;
/// use std::fs::File;
/// use std::io::Write;
/// use flate2::{Compression, write::GzEncoder};
/// use chromsize::get_sizes;
///
/// File::create("test.fa").unwrap().write_all(b">chr1 description\nATGC\n>chr2\nGCTA").unwrap();
/// let sizes_plain = get_sizes(PathBuf::from("test.fa"), false).unwrap();
/// assert!(sizes_plain.contains(&("chr1 description".to_string(), 4)));
/// assert!(sizes_plain.contains(&("chr2".to_string(), 4)));
/// std::fs::remove_file("test.fa").unwrap();
///
/// let mut encoder = GzEncoder::new(File::create("test.fa.gz").unwrap(), Compression::default());
/// encoder.write_all(b">chrX another desc\nNNNN\n>chrY\nAAAA").unwrap();
/// encoder.finish().unwrap();
/// let sizes_gz = get_sizes(PathBuf::from("test.fa.gz"), true).unwrap();
/// assert!(sizes_gz.contains(&("chrX".to_string(), 4)));
/// assert!(sizes_gz.contains(&("chrY".to_string(), 4)));
/// std::fs::remove_file("test.fa.gz").unwrap();
/// ```
pub fn get_sizes<T: AsRef<Path> + Debug>(
    fasta: T,
    accession_only: bool,
) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let path = fasta.as_ref();
    let ext = path.extension().unwrap();
    let file = File::open(path)?;

    let lines = match ext.to_str().unwrap() {
        "gz" => with_gz(&file, accession_only)?,
        "fa" | "fasta" | "fna" => raw(&file, accession_only)?,
        _ => panic!("ERROR: Not a fasta. Wrong file format!"),
    };

    Ok(lines)
}

/// Reads a plain (non-gzipped) FASTA file using memory mapping and extracts chromosome sizes.
///
/// This internal helper function maps the entire file into memory for efficient access,
/// and then processes the byte slice using `chromsize` to get chromosome names and lengths.
///
/// # Arguments
/// * `file` - A reference to the opened `File` object.
/// * `accession_only` - A boolean indicating whether to extract only the accession
///                      part of the FASTA header.
///
/// # Returns
/// A `Result` containing a `Vec` of `(String, u64)` tuples, or a `Box<dyn Error>` if
/// memory mapping fails or `chromsize` encounters an error.
///
/// # Safety
/// This function uses `unsafe { Mmap::map(file)? }` to memory-map the file.
/// This is safe as long as the file descriptor remains valid for the lifetime of the `Mmap` object.
///
/// # Example
/// ```rust, ignore
/// use std::fs::File;
/// use std::io::Write;
/// use chromsize::raw;
///
/// File::create("temp.fa").unwrap().write_all(b">chrA\nAAAA\n>chrB\nTTTT").unwrap();
/// let file = File::open("temp.fa").unwrap();
/// let sizes = raw(&file, false).unwrap();
/// assert!(sizes.contains(&("chrA".to_string(), 4)));
/// assert!(sizes.contains(&("chrB".to_string(), 4)));
/// std::fs::remove_file("temp.fa").unwrap();
/// ```
pub fn raw(file: &File, accession_only: bool) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let mmap = unsafe { Mmap::map(file)? };
    let lines = chromsize(&mmap, accession_only)?;

    Ok(lines)
}

/// Reads a gzipped FASTA file, decompresses it into a buffer, and extracts chromosome sizes.
///
/// This internal helper function memory-maps the gzipped file, uses `MultiGzDecoder`
/// to decompress its content into an in-memory buffer, and then processes the buffer
/// using `chromsize` to get chromosome names and lengths.
///
/// # Arguments
/// * `file` - A reference to the opened gzipped `File` object.
/// * `accession_only` - A boolean indicating whether to extract only the accession
///                      part of the FASTA header.
///
/// # Returns
/// A `Result` containing a `Vec` of `(String, u64)` tuples, or a `Box<dyn Error>` if
/// memory mapping fails, decompression fails, or `chromsize` encounters an error.
///
/// # Safety
/// This function uses `unsafe { Mmap::map(file)? }` to memory-map the file.
/// This is safe as long as the file descriptor remains valid for the lifetime of the `Mmap` object.
///
/// # Example
/// ```rust, ignore
/// use std::fs::File;
/// use std::io::Write;
/// use flate2::{Compression, write::GzEncoder};
/// use chromsize::with_gz;
///
/// let mut encoder = GzEncoder::new(File::create("temp.fa.gz").unwrap(), Compression::default());
/// encoder.write_all(b">seq1\nGCAT\n>seq2\nTAGC").unwrap();
/// encoder.finish().unwrap();
///
/// let file = File::open("temp.fa.gz").unwrap();
/// let sizes = with_gz(&file, false).unwrap();
/// assert!(sizes.contains(&("seq1".to_string(), 4)));
/// assert!(sizes.contains(&("seq2".to_string(), 4)));
/// std::fs::remove_file("temp.fa.gz").unwrap();
/// ```
fn with_gz(file: &File, accession_only: bool) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let mmap = unsafe { Mmap::map(file)? };
    let mut decoder = MultiGzDecoder::new(&mmap[..]);

    let mut buffer = Vec::with_capacity(100 * 1024 * 1024); // 100MB buffer
    decoder.read_to_end(&mut buffer)?;

    let lines = chromsize(&buffer, accession_only)?;

    Ok(lines)
}

/// Parses a byte slice (representing FASTA content) to extract chromosome names and lengths.
///
/// This internal helper function processes the raw byte data. It splits the data
/// by FASTA header markers (`>`), extracts the chromosome name (optionally `accession_only`),
/// and calculates the total length of the sequence lines for each chromosome.
/// It uses `rayon` for parallel processing of chunks for performance.
///
/// # Arguments
/// * `data` - The byte slice containing the FASTA content.
/// * `accession_only` - A boolean indicating whether to extract only the accession
///                      part of the FASTA header.
///
/// # Returns
/// A `Result` containing a `Vec` of `(String, u64)` tuples, or a `Box<dyn Error>` if
/// UTF-8 conversion fails (though `unsafe { from_utf8_unchecked }` is used here,
/// implying an expectation of valid UTF-8).
///
/// # Safety
/// This function uses `unsafe { std::str::from_utf8_unchecked(...) }`. This is safe
/// *if and only if* the byte slices being converted are guaranteed to be valid UTF-8.
/// In the context of FASTA files, sequence data and header information are typically
/// ASCII, making this assumption reasonable for common use cases.
///
/// # Example
/// ```rust
/// use chromsize::chromsize;
///
/// // Example with full headers
/// let data1 = b">chr1 description one\nATGCATGC\n>chr2 description two\nGGCC";
/// let sizes1 = chromsize(data1, false).unwrap();
/// assert!(sizes1.contains(&("chr1 description one".to_string(), 8)));
/// assert!(sizes1.contains(&("chr2 description two".to_string(), 4)));
///
/// // Example with accession only
/// let data2 = b">chrA desc A\nTTTT\n>chrB desc B\nCCCC";
/// let sizes2 = chromsize(data2, true).unwrap();
/// assert!(sizes2.contains(&("chrA".to_string(), 4)));
/// assert!(sizes2.contains(&("chrB".to_string(), 4)));
///
/// // Example with empty lines or no sequence
/// let data3 = b">empty_chr\n\n>another_chr\nABC";
/// let sizes3 = chromsize(data3, false).unwrap();
/// assert!(sizes3.contains(&("empty_chr".to_string(), 0)));
/// assert!(sizes3.contains(&("another_chr".to_string(), 3)));
/// ```
fn chromsize(data: &[u8], accession_only: bool) -> Result<Vec<(String, u64)>, Box<dyn Error>> {
    let lines = data
        .par_split(|&c| c == b'>')
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            let mut totals = 0u64;
            let stop = memchr::memchr(b'\n', chunk).unwrap_or(0);
            let name_stop = match accession_only {
                true => memchr::memchr(b' ', &chunk[..stop]).unwrap_or(stop),
                false => stop,
            };
            let chr = unsafe {
                std::str::from_utf8_unchecked(&chunk[..name_stop])
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

/// Writes chromosome sizes (name and length) to an output file.
///
/// This function takes a vector of `(String, u64)` tuples and writes each
/// pair as a tab-separated line to the specified output file. It uses a
/// `BufWriter` for efficient buffered writing. Records with empty chromosome
/// names and zero length are explicitly skipped.
///
/// # Type Parameters
/// * `T` - The type of the output path, which must implement `AsRef<Path>` and `Debug`.
///
/// # Arguments
/// * `sizes` - A `Vec` of `(String, u64)` tuples representing chromosome names and their lengths.
/// * `out` - The path to the output file where the sizes will be written.
///
/// # Panics
/// * If the output file cannot be created.
/// * If an I/O error occurs during writing to the file.
///
/// # Example
/// ```rust, ignore
/// use std::fs::{File, read_to_string};
/// use std::path::PathBuf;
/// use chromsize::writer;
///
/// let mut sizes_data = Vec::new();
/// sizes_data.push(("chr1".to_string(), 1000));
/// sizes_data.push(("chr2".to_string(), 500));
/// sizes_data.push(("".to_string(), 0)); // This record should be skipped
///
/// writer(sizes_data, PathBuf::from("chrom_sizes.txt"));
///
/// let content = read_to_string("chrom_sizes.txt").unwrap();
/// assert_eq!(content, "chr1\t1000\nchr2\t500\n");
/// std::fs::remove_file("chrom_sizes.txt").unwrap();
/// ```
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
        if v == &0 && k.is_empty() {
            // INFO: skip zero-length chromosomes and empty names
            // INFO: see github.com/alejandrogzi/chromsize/pull/3
            continue;
        }

        writeln!(writer, "{}\t{}", k, v).unwrap();
    }
}
