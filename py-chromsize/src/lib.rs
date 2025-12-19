use chromsize;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyfunction]
fn get_chromsizes(py: Python, sequence: PyObject) -> PyResult<Vec<(String, u64)>> {
    let sequence = PathBuf::from(sequence.extract::<String>(py)?);

    chromsize::get_sizes(&sequence)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
}

#[pyfunction]
fn write_chromsizes(py: Python, sequence: PyObject, output: PyObject) -> PyResult<String> {
    let sequence = PathBuf::from(sequence.extract::<String>(py)?);
    let output = PathBuf::from(output.extract::<String>(py)?);

    let sizes = chromsize::get_sizes(&sequence)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

    chromsize::writer(&sizes, &output)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

    Ok(format!("Chromosome sizes written to {}", output.display()))
}

#[pymodule]
#[pyo3(name = "chromsize")]
fn py_chromsize(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_chromsizes, m)?)?;
    m.add_function(wrap_pyfunction!(write_chromsizes, m)?)?;
    Ok(())
}
