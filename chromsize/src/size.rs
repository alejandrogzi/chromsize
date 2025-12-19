//! chromsize
//! Alejandro Gonzales-Irribarren, 2024
//!
//! `chromsize` is a utility designed to extract chromosome names
//! and their corresponding lengths from FASTA and 2bit files. It supports
//! both plain and gzipped FASTA formats [and 2bit] and offers an option to
//! include only the accession ID from the FASTA headers.

use flate2::read::MultiGzDecoder;
use memmap2::Mmap;
use rayon::prelude::*;
use std::{
    fmt,
    fmt::Debug,
    fs::File,
    io::{self, BufWriter, Read, Write},
    path::Path,
};

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];
const TWOBIT_MAGIC: [u8; 4] = [0x1a, 0x41, 0x27, 0x43];
const TWOBIT_MAGIC_REV: [u8; 4] = [0x43, 0x27, 0x41, 0x1a];

/// Error types for the chromsize application.
///
/// This enum represents all possible error conditions that can occur
/// during sequence processing and chromosome size calculation.
#[derive(Debug)]
pub enum ChromsizeError {
    /// I/O related errors from file operations
    Io(io::Error),
    /// Empty input data (no content to process)
    EmptyInput,
    /// Invalid input format or unsupported content
    InvalidInput(String),
    /// Invalid FASTA format with descriptive message
    InvalidFasta(String),
}

impl fmt::Display for ChromsizeError {
    /// Formats the error for display purposes.
    ///
    /// Provides human-readable error messages for each error variant.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChromsizeError::Io(err) => write!(f, "I/O error: {}", err),
            ChromsizeError::EmptyInput => write!(f, "Input is empty"),
            ChromsizeError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ChromsizeError::InvalidFasta(msg) => write!(f, "Invalid FASTA: {}", msg),
        }
    }
}

impl std::error::Error for ChromsizeError {
    /// Returns the underlying error source for error chaining.
    ///
    /// Only IO errors have an underlying source, other errors are standalone.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChromsizeError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ChromsizeError {
    /// Converts I/O errors to ChromsizeError.
    ///
    /// This enables the `?` operator for I/O operations throughout the codebase.
    fn from(err: io::Error) -> Self {
        ChromsizeError::Io(err)
    }
}

/// Represents different types of input data storage.
///
/// This enum allows handling both memory-mapped files and owned buffers,
/// enabling efficient processing of different input sources.
enum InputData {
    /// Memory-mapped file data (zero-copy for regular files)
    Mmap(Mmap),
    /// Owned byte buffer (for stdin or decompressed data)
    Owned(Vec<u8>),
}

enum InputFormat {
    Fasta,
    TwoBit,
}

impl AsRef<[u8]> for InputData {
    /// Provides read access to the underlying byte data.
    ///
    /// This enables uniform processing regardless of whether the data
    /// comes from a memory-mapped file or an owned buffer.
    fn as_ref(&self) -> &[u8] {
        match self {
            InputData::Mmap(m) => m.as_ref(),
            InputData::Owned(b) => b.as_slice(),
        }
    }
}

/// Extracts chromosome sizes from a sequence file.
///
/// This function processes a sequence file (from file path or stdin) and returns
/// a vector of tuples containing chromosome names and their corresponding sizes.
/// Input format is detected by content (FASTA header or 2bit signature).
///
/// # Arguments
///
/// * `sequence` - Path to a FASTA/2bit file or "-" for stdin
///
/// # Returns
///
/// Vector of (chromosome_name, size) pairs
///
/// # Examples
///
/// ```ignore
/// let sizes = get_sizes("genome.fa")?;
/// // sizes: [("chr1", 248956422), ("chr2", 242193529), ...]
/// ```
pub fn get_sizes<T: AsRef<Path> + Debug>(
    sequence: T,
) -> Result<Vec<(String, u64)>, ChromsizeError> {
    let path = sequence.as_ref();
    let data = if is_stdin(path) {
        from_stdin()?
    } else {
        from_file(path)?
    };

    sizes_from_bytes(data.as_ref())
}

/// Checks if the path represents stdin input.
///
/// Returns true if the path is "-" which conventionally means read from stdin.
/// Checks if the path represents stdin input.
///
/// Returns true if the path is "-" which conventionally means read from stdin.
fn is_stdin(path: &Path) -> bool {
    path == Path::new("-")
}

/// Reads data from standard input with optional gzip decompression.
///
/// This function reads all data from stdin, detects if it's gzip-compressed,
/// and decompresses it if necessary. Returns the data in an owned buffer.
///
/// # Returns
///
/// InputData::Owned containing the raw or decompressed input data
/// Reads data from standard input with optional gzip decompression.
///
/// This function reads all data from stdin, detects if it's gzip-compressed,
/// and decompresses it if necessary. Returns the data in an owned buffer.
///
/// # Returns
///
/// InputData::Owned containing the raw or decompressed input data
fn from_stdin() -> Result<InputData, ChromsizeError> {
    let mut buffer = Vec::with_capacity(1024 * 1024);
    let mut handle = io::stdin().lock();
    handle.read_to_end(&mut buffer)?;

    if buffer.is_empty() {
        return Err(ChromsizeError::EmptyInput);
    }

    if is_gzip(&buffer) {
        let decompressed = decompress_gzip(&buffer)?;

        if decompressed.is_empty() {
            return Err(ChromsizeError::EmptyInput);
        }

        Ok(InputData::Owned(decompressed))
    } else {
        Ok(InputData::Owned(buffer))
    }
}

/// Reads data from a file with memory mapping and optional gzip decompression.
///
/// This function attempts to memory-map the file for efficient access.
/// If the file is gzip-compressed, it decompresses the entire content
/// into an owned buffer instead.
///
/// # Arguments
///
/// * `path` - Path to the input file
///
/// # Returns
///
/// InputData::Mmap for uncompressed files, InputData::Owned for compressed files
/// Reads data from a file with memory mapping and optional gzip decompression.
///
/// This function attempts to memory-map the file for efficient access.
/// If the file is gzip-compressed, it decompresses the entire content
/// into an owned buffer instead.
///
/// # Arguments
///
/// * `path` - Path to the input file
///
/// # Returns
///
/// InputData::Mmap for uncompressed files, InputData::Owned for compressed files
fn from_file(path: &Path) -> Result<InputData, ChromsizeError> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    if mmap.is_empty() {
        return Err(ChromsizeError::EmptyInput);
    }

    if is_gzip(&mmap) {
        let decompressed = decompress_gzip(&mmap)?;
        if decompressed.is_empty() {
            return Err(ChromsizeError::EmptyInput);
        }
        Ok(InputData::Owned(decompressed))
    } else {
        Ok(InputData::Mmap(mmap))
    }
}

/// Reads 2bit data and extracts chromosome sizes.
///
/// This internal helper function opens and reads 2bit data using the `twobit` crate,
/// then extracts chromosome names and their corresponding sizes. The function converts
/// chromosome sizes from `usize` to `u64` for consistency with other chromsize functions.
///
/// # Arguments
/// * `twobit` - Raw 2bit data to be processed.
///
/// # Returns
/// A `Result` containing a `Vec` of `(String, u64)` tuples representing chromosome names
/// and their sizes in base pairs, or an error if the data cannot be parsed.
///
fn from_2bit(twobit: &[u8]) -> Result<Vec<(String, u64)>, ChromsizeError> {
    let genome = twobit::TwoBitFile::from_buf(twobit)
        .map_err(|e| ChromsizeError::InvalidInput(format!("Invalid 2bit data: {e}")))?;

    let cs = genome
        .chrom_names()
        .into_iter()
        .zip(genome.chrom_sizes())
        .map(|(chr, size)| (chr, size as u64))
        .collect();

    Ok(cs)
}

/// Processes raw bytes to extract chromosome sizes by detecting format.
///
/// This function acts as a dispatcher that determines the input format
/// (FASTA or 2bit) and routes the data to the appropriate processor.
/// It returns an error if the format is not recognized.
///
/// # Arguments
///
/// * `data` - Raw byte data to be processed
///
/// # Returns
///
/// Vector of (chromosome_name, size) pairs or error for unsupported format
///
/// # Examples
///
/// ```ignore
/// let data = b">chr1\nATGC\n>chr2\nGCTA";
/// let sizes = sizes_from_bytes(data)?;
/// // sizes: [("chr1", 4), ("chr2", 4)]
/// ```
fn sizes_from_bytes(data: &[u8]) -> Result<Vec<(String, u64)>, ChromsizeError> {
    match sniff_format(data) {
        Some(InputFormat::TwoBit) => from_2bit(data),
        Some(InputFormat::Fasta) => chromsize(data),
        None => Err(ChromsizeError::InvalidInput(
            "Input format not recognized (expected FASTA or 2bit)".to_string(),
        )),
    }
}

/// Detects the input format by examining file magic bytes and patterns.
///
/// This function identifies whether the data is in 2bit format (by checking
/// the 4-byte magic signature) or FASTA format (by checking for the '>' header
/// character). Returns None for unrecognized formats.
///
/// # Arguments
///
/// * `data` - Raw byte data to examine for format identification
///
/// # Returns
///
/// Some(InputFormat) if format is recognized, None otherwise
fn sniff_format(data: &[u8]) -> Option<InputFormat> {
    if data.len() >= TWOBIT_MAGIC.len()
        && (data[..TWOBIT_MAGIC.len()] == TWOBIT_MAGIC
            || data[..TWOBIT_MAGIC_REV.len()] == TWOBIT_MAGIC_REV)
    {
        return Some(InputFormat::TwoBit);
    }

    if data.first() == Some(&b'>') {
        return Some(InputFormat::Fasta);
    }

    None
}

/// Detects if data is gzip-compressed by checking magic bytes.
///
/// Gzip files start with the magic bytes 0x1f 0x8b.
///
/// # Arguments
///
/// * `bytes` - Byte slice to check
///
/// # Returns
///
/// true if the data appears to be gzip-compressed
/// Detects if data is gzip-compressed by checking magic bytes.
///
/// Gzip files start with the magic bytes 0x1f 0x8b.
///
/// # Arguments
///
/// * `bytes` - Byte slice to check
///
/// # Returns
///
/// true if the data appears to be gzip-compressed
fn is_gzip(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == GZIP_MAGIC[0] && bytes[1] == GZIP_MAGIC[1]
}

/// Decompresses gzip data into a byte vector.
///
/// This function handles potentially large gzip files efficiently
/// by using a reasonably sized initial buffer.
///
/// # Arguments
///
/// * `data` - Compressed gzip data
///
/// # Returns
///
/// Decompressed data in a new Vec<u8>
/// Decompresses gzip data into a byte vector.
///
/// This function handles potentially large gzip files efficiently
/// by using a reasonably sized initial buffer.
///
/// # Arguments
///
/// * `data` - Compressed gzip data
///
/// # Returns
///
/// Decompressed data in a new Vec<u8>
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, ChromsizeError> {
    let mut decoder = MultiGzDecoder::new(data);
    let mut buffer = Vec::with_capacity(8 * 1024 * 1024);
    decoder.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Processes FASTA data to extract chromosome sizes in parallel.
///
/// This is the core processing function that validates FASTA format
/// and uses parallel processing to handle multiple chromosomes efficiently.
///
/// # Arguments
///
/// * `data` - Raw FASTA data as bytes
///
/// # Returns
///
/// Vector of (chromosome_name, size) pairs
/// Processes FASTA data to extract chromosome sizes in parallel.
///
/// This is the core processing function that validates FASTA format
/// and uses parallel processing to handle multiple chromosomes efficiently.
///
/// # Arguments
///
/// * `data` - Raw FASTA data as bytes
///
/// # Returns
///
/// Vector of (chromosome_name, size) pairs
fn chromsize(data: &[u8]) -> Result<Vec<(String, u64)>, ChromsizeError> {
    if data.is_empty() {
        return Err(ChromsizeError::EmptyInput);
    }

    if data[0] != b'>' {
        return Err(ChromsizeError::InvalidFasta(
            "Input does not start with '>'".to_string(),
        ));
    }

    data.par_split(|&c| c == b'>')
        .filter(|chunk| !chunk.is_empty())
        .map(process_record)
        .collect()
}

/// Processes a single FASTA record to extract header and count sequence length.
///
/// This function parses a FASTA record (header + sequence), validates the format,
/// and counts the total number of valid sequence characters while enforcing
/// strict FASTA format compliance.
///
/// # Arguments
///
/// * `chunk` - FASTA record data without the leading '>' character
///
/// # Returns
///
/// Tuple of (header_string, sequence_length)
/// Processes a single FASTA record to extract header and count sequence length.
///
/// This function parses a FASTA record (header + sequence), validates the format,
/// and counts the total number of valid sequence characters while enforcing
/// strict FASTA format compliance.
///
/// # Arguments
///
/// * `chunk` - FASTA record data without the leading '>' character
///
/// # Returns
///
/// Tuple of (header_string, sequence_length)
fn process_record(chunk: &[u8]) -> Result<(String, u64), ChromsizeError> {
    let Some(stop) = memchr::memchr(b'\n', chunk) else {
        return Err(ChromsizeError::InvalidFasta(
            "Record header is not terminated by a newline".to_string(),
        ));
    };

    let header = std::str::from_utf8(&chunk[..stop])
        .map_err(|_| ChromsizeError::InvalidFasta("Record header is not UTF-8".to_string()))?
        .trim();

    if header.is_empty() {
        return Err(ChromsizeError::InvalidFasta(
            "Record has an empty header".to_string(),
        ));
    }

    let data = &chunk[stop + 1..];

    if memchr::memchr2(b' ', b'\t', data).is_some() {
        return Err(ChromsizeError::InvalidFasta(
            "Record contains whitespace inside sequence data".to_string(),
        ));
    }

    if memchr::memchr(b'>', data).is_some() {
        return Err(ChromsizeError::InvalidFasta(
            "Record contains '>' inside sequence data".to_string(),
        ));
    }

    if data.iter().any(|&b| match b {
        b'\n' | b'\r' => false,
        0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f => true,
        0x20..=0x7e => false,
        _ => true,
    }) {
        return Err(ChromsizeError::InvalidFasta(
            "Record contains invalid control or non-ASCII characters".to_string(),
        ));
    }

    let count_newlines = bytecount::count(data, b'\n') as u64;
    let count_cr = bytecount::count(data, b'\r') as u64;
    let totals = data.len() as u64 - count_newlines - count_cr;

    Ok((header.to_string(), totals))
}

/// Writes chromosome sizes to a tab-delimited file.
///
/// This function takes the calculated chromosome sizes and writes them
/// to the specified output file in a standard two-column format:
/// chromosome_name<TAB>size
///
/// # Arguments
///
/// * `sizes` - Vector of (chromosome_name, size) tuples
/// * `out` - Output file path
///
/// # Examples
///
/// ```ignore
/// let sizes = vec![("chr1".to_string(), 248956422), ("chr2".to_string(), 242193529)];
/// writer(&sizes, "chrom.sizes")?;
/// // Output file contains:
/// // chr1    248956422
/// // chr2    242193529
/// ```
pub fn writer<T>(sizes: &[(String, u64)], out: T) -> Result<(), ChromsizeError>
where
    T: AsRef<Path> + Debug,
{
    let file = File::create(out)?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    for (k, v) in sizes.iter() {
        writeln!(writer, "{}\t{}", k, v)?;
    }

    Ok(())
}
