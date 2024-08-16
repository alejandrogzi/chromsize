use chromsize;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyfunction]
fn get_chromsizes(py: Python, fasta: PyObject) -> PyResult<Vec<(String, u64)>> {
    let fasta = PathBuf::from(fasta.extract::<String>(py)?);

    let sizes = chromsize::get_sizes(&fasta);
    sizes.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
}

#[pyfunction]
fn write_chromsizes(py: Python, fasta: PyObject, output: PyObject) -> PyResult<String> {
    let fasta = PathBuf::from(fasta.extract::<String>(py)?);
    let output = PathBuf::from(output.extract::<String>(py)?);

    let sizes = chromsize::get_sizes(&fasta);
    if let Ok(sizes) = sizes {
        chromsize::writer(sizes, &output);
        Ok(format!("Chromosome sizes written to {}", output.display()))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Failed to get chromosome sizes",
        ))
    }
}

#[pymodule]
#[pyo3(name = "chromsize")]
fn py_chromsize(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_chromsizes, m)?)?;
    m.add_function(wrap_pyfunction!(write_chromsizes, m)?)?;
    Ok(())
}
