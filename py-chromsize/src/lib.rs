use chromsize;
use pyo3::prelude::*;
use std::path::PathBuf;

/// Python function to get chromosome sizes from a FASTA file.
///
/// This function serves as a Python binding for the Rust `get_sizes` function.
/// It reads a FASTA file (plain or gzipped) and returns a list of tuples,
/// where each tuple contains the chromosome name (String) and its length (u64).
///
/// # Arguments
/// * `py` - Python interpreter instance.
/// * `fasta` - The path to the FASTA file (as a Python string, converted to `PathBuf`).
/// * `accession_only` - (Optional) A boolean. If `True`, only the accession ID
///                      part of the header (before the first blank space) is kept.
///                      Defaults to `False`.
///
/// # Returns
/// A `PyResult` containing a `Vec<(String, u64)>` which will be converted to a Python list
/// of tuples `(str, int)`.
///
/// # Errors
/// Returns a `PyValueError` if there's an issue reading the FASTA file,
/// parsing its content, or if the file format is unsupported.
///
/// # Python Example
/// ```python
/// import chromsize
/// import os
///
/// # Create a dummy FASTA file for demonstration
/// with open("test_fasta.fa", "w") as f:
///     f.write(">chr1 desc1\nATGC\n>chr2\nGGGG")
///
/// # Get chromosome sizes
/// sizes = chromsize.get_chromsizes("test_fasta.fa")
/// print(sizes)
/// # Expected output: [('chr1 desc1', 4), ('chr2', 4)]
///
/// # Get chromosome sizes with accession_only
/// sizes_acc = chromsize.get_chromsizes("test_fasta.fa", accession_only=True)
/// print(sizes_acc)
/// # Expected output: [('chr1', 4), ('chr2', 4)]
///
/// # Clean up the dummy file
/// os.remove("test_fasta.fa")
/// ```
#[pyfunction]
#[pyo3(signature = (fasta, accession_only=false))]
fn get_chromsizes(
    py: Python,
    fasta: PyObject,
    accession_only: bool,
) -> PyResult<Vec<(String, u64)>> {
    let fasta = PathBuf::from(fasta.extract::<String>(py)?);

    let sizes = chromsize::get_sizes(&fasta, accession_only);
    sizes.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
}

/// Python function to write chromosome sizes to an output file.
///
/// This function reads chromosome sizes from a FASTA file and writes them
/// to a specified output file in a tab-separated format (name<tab>length).
/// It leverages the Rust `get_sizes` and `writer` functions.
///
/// # Arguments
/// * `py` - Python interpreter instance.
/// * `fasta` - The path to the input FASTA file (as a Python string).
/// * `output` - The path to the output file where sizes will be written (as a Python string).
/// * `accession_only` - (Optional) A boolean. If `True`, only the accession ID
///                      part of the header is kept when reading the FASTA. Defaults to `False`.
///
/// # Returns
/// A `PyResult` containing a `String` message indicating success,
/// typically the path to the written file.
///
/// # Errors
/// Returns a `PyValueError` if there's an issue reading the FASTA file
/// or writing to the output file.
///
/// # Python Example
/// ```python
/// import chromsize
/// import os
///
/// # Create a dummy FASTA file
/// with open("input_write.fa", "w") as f:
///     f.write(">chrA long_desc\nATCGC\n>chrB\nTA")
///
/// output_file = "output_sizes.txt"
///
/// # Write chromosome sizes to file
/// result_message = chromsize.write_chromsizes("input_write.fa", output_file)
/// print(result_message)
/// # Expected output: "Chromosome sizes written to output_sizes.txt"
///
/// # Verify content
/// with open(output_file, "r") as f:
///     content = f.read()
/// print(content)
/// # Expected content:
/// # chrA long_desc    5
/// # chrB    2
///
/// # Clean up dummy files
/// os.remove("input_write.fa")
/// os.remove(output_file)
/// ```
#[pyfunction]
#[pyo3(signature = (fasta, output, accession_only=false))]
fn write_chromsizes(
    py: Python,
    fasta: PyObject,
    output: PyObject,
    accession_only: bool,
) -> PyResult<String> {
    let fasta = PathBuf::from(fasta.extract::<String>(py)?);
    let output = PathBuf::from(output.extract::<String>(py)?);

    let sizes = chromsize::get_sizes(&fasta, accession_only);
    if let Ok(sizes) = sizes {
        chromsize::writer(sizes, &output);
        Ok(format!("Chromosome sizes written to {}", output.display()))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Failed to get chromosome sizes",
        ))
    }
}

/// A PyO3 module for `chromsize`
///
/// This module exposes Rust functions to Python, allowing Python users
/// to efficiently retrieve and write chromosome sizes from FASTA files.
///
/// # Functions
/// * `get_chromsizes(fasta: str, accession_only: bool = False) -> list[tuple[str, int]]`:
///   Retrieves chromosome names and lengths from a FASTA file.
/// * `write_chromsizes(fasta: str, output: str, accession_only: bool = False) -> str`:
///   Writes chromosome names and lengths from a FASTA file to a specified output file.
///
/// # Python Example (To be used in a Python file after installing the Rust library)
/// ```python
/// # Save this as a Python file (e.g., example.py)
/// # Make sure your Rust library is compiled and installed (e.g., using `maturin develop` or `pip install .`)
/// import chromsize
/// import os
///
/// # Create a dummy FASTA file for demonstration
/// with open("demo.fa", "w") as f:
///     f.write(">chrX details for X\nAAATT\n>chrY\nGGCCC")
///
/// # Example 1: Get sizes directly
/// sizes_list = chromsize.get_chromsizes("demo.fa", accession_only=True)
/// print(f"Retrieved sizes: {sizes_list}")
/// # Expected: Retrieved sizes: [('chrX', 5), ('chrY', 5)]
///
/// # Example 2: Write sizes to a file
/// output_file_path = "demo_chromsizes.txt"
/// message = chromsize.write_chromsizes("demo.fa", output_file_path)
/// print(message)
/// # Expected: Chromosome sizes written to demo_chromsizes.txt
///
/// # Verify the content of the written file
/// with open(output_file_path, "r") as f:
///     file_content = f.read()
/// print(f"Content of {output_file_path}:\n{file_content}")
/// # Expected content:
/// # chrX details for X    5
/// # chrY    5
///
/// # Clean up dummy files
/// os.remove("demo.fa")
/// os.remove(output_file_path)
/// ```
#[pymodule]
#[pyo3(name = "chromsize")]
fn py_chromsize(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_chromsizes, m)?)?;
    m.add_function(wrap_pyfunction!(write_chromsizes, m)?)?;
    Ok(())
}
