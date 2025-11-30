use pyo3::prelude::*;
use pyo3::types::PyDict;
use rquickjs::{Context, Function, Runtime};
use std::cell::RefCell;
use std::collections::HashMap;

const JS_CODE: &str = include_str!("../js/tex2typst.bundle.js");

/// Internal converter instance
struct ConverterInstance {
    _rt: Runtime,
    ctx: Context,
}

impl ConverterInstance {
    fn new() -> PyResult<Self> {
        let rt = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let ctx = Context::full(&rt)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        ctx.with(|ctx| {
            ctx.eval::<(), _>(JS_CODE).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("JS Load Error: {}", e))
            })
        })?;

        Ok(ConverterInstance { _rt: rt, ctx })
    }

    fn tex2typst(
        &self,
        tex: &str,
        options: Option<HashMap<String, serde_json::Value>>,
    ) -> PyResult<String> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();

            let func: Function = globals.get("tex2typst").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'tex2typst' not found.",
                )
            })?;

            let result: String = if let Some(opts) = options {
                // Create JavaScript object from options
                let js_options = ctx
                    .json_parse(serde_json::to_string(&opts).unwrap())
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Options parse failed: {}",
                            e
                        ))
                    })?;

                func.call((tex, js_options)).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        e
                    ))
                })?
            } else {
                func.call((tex,)).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        e
                    ))
                })?
            };

            Ok(result)
        })
    }

    fn typst2tex(
        &self,
        typst: &str,
        options: Option<HashMap<String, serde_json::Value>>,
    ) -> PyResult<String> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();

            let func: Function = globals.get("typst2tex").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'typst2tex' not found.",
                )
            })?;

            let result: String = if let Some(opts) = options {
                // Create JavaScript object from options
                let js_options = ctx
                    .json_parse(serde_json::to_string(&opts).unwrap())
                    .map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Options parse failed: {}",
                            e
                        ))
                    })?;

                func.call((typst, js_options)).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        e
                    ))
                })?
            } else {
                func.call((typst,)).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        e
                    ))
                })?
            };

            Ok(result)
        })
    }
}

// Thread-local lazy singleton for module-level functions
thread_local! {
    static THREAD_CONVERTER: RefCell<Option<ConverterInstance>> = const { RefCell::new(None) };
}

fn get_thread_converter() -> PyResult<()> {
    THREAD_CONVERTER.with(|converter| {
        if converter.borrow().is_none() {
            *converter.borrow_mut() = Some(ConverterInstance::new()?);
        }
        Ok(())
    })
}

/// Convert Python dict to HashMap<String, serde_json::Value>
fn pydict_to_hashmap(py_dict: &Bound<PyDict>) -> PyResult<HashMap<String, serde_json::Value>> {
    let mut map = HashMap::new();

    for (key, value) in py_dict.iter() {
        let key_str: String = key.extract()?;

        // Convert Python values to serde_json::Value
        let json_value = if let Ok(b) = value.extract::<bool>() {
            serde_json::Value::Bool(b)
        } else if let Ok(i) = value.extract::<i64>() {
            serde_json::Value::Number(i.into())
        } else if let Ok(f) = value.extract::<f64>() {
            serde_json::Value::Number(serde_json::Number::from_f64(f).ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid float value")
            })?)
        } else if let Ok(s) = value.extract::<String>() {
            serde_json::Value::String(s)
        } else if value.is_instance_of::<PyDict>() {
            // Handle nested dict (for customTexMacros)
            let d = value.cast_into::<PyDict>()?;
            let nested = pydict_to_hashmap(&d)?;
            serde_json::to_value(nested).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize nested dict: {}",
                    e
                ))
            })?
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Unsupported type for key '{}'",
                key_str
            )));
        };

        map.insert(key_str, json_value);
    }

    Ok(map)
}

/// Convert LaTeX/TeX math to Typst format.
///
/// Uses a thread-local lazy singleton - the converter is initialized only on the
/// first call within each thread, avoiding import-time overhead.
///
/// Args:
///     tex: LaTeX/TeX math string to convert
///     options: Optional dictionary of conversion options
///
/// Returns:
///     Converted Typst string
#[pyfunction]
#[pyo3(signature = (tex, options=None))]
fn tex2typst(tex: String, options: Option<&Bound<PyDict>>) -> PyResult<String> {
    get_thread_converter()?;

    let opts = if let Some(py_dict) = options {
        Some(pydict_to_hashmap(py_dict)?)
    } else {
        None
    };

    THREAD_CONVERTER.with(|converter| converter.borrow().as_ref().unwrap().tex2typst(&tex, opts))
}

/// Convert Typst math to LaTeX/TeX format.
///
/// Uses a thread-local lazy singleton - the converter is initialized only on the
/// first call within each thread, avoiding import-time overhead.
///
/// Args:
///     typst: Typst math string to convert
///     options: Optional dictionary of conversion options
///
/// Returns:
///     Converted LaTeX/TeX string
#[pyfunction]
#[pyo3(signature = (typst, options=None))]
fn typst2tex(typst: String, options: Option<&Bound<PyDict>>) -> PyResult<String> {
    get_thread_converter()?;

    let opts = if let Some(py_dict) = options {
        Some(pydict_to_hashmap(py_dict)?)
    } else {
        None
    };

    THREAD_CONVERTER.with(|converter| converter.borrow().as_ref().unwrap().typst2tex(&typst, opts))
}

#[pymodule]
#[pyo3(name = "tex2typst")]
fn tex2typst_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tex2typst, m)?)?;
    m.add_function(wrap_pyfunction!(typst2tex, m)?)?;
    Ok(())
}
