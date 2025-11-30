use pyo3::prelude::*;
use pyo3::types::PyDict;
use rquickjs::{CatchResultExt, CaughtError, Context, Function, Runtime};
use std::cell::RefCell;
use std::collections::HashMap;

const JS_CODE: &str = include_str!("../js/tex2typst.bundle.js");

/// Format a QuickJS exception with detailed error information
fn format_js_exception(error: CaughtError) -> String {
    match error {
        CaughtError::Exception(exception) => {
            let message = exception
                .message()
                .unwrap_or_else(|| "Unknown error".to_string());

            if let Some(stack) = exception.stack() {
                format!("{}\nStack trace:\n{}", message, stack)
            } else {
                message
            }
        }
        CaughtError::Error(err) => err.to_string(),
        CaughtError::Value(val) => format!("JavaScript error: {:?}", val),
    }
}

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

                func.call((tex, js_options)).catch(&ctx).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        format_js_exception(e)
                    ))
                })?
            } else {
                func.call((tex,)).catch(&ctx).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        format_js_exception(e)
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

                func.call((typst, js_options)).catch(&ctx).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        format_js_exception(e)
                    ))
                })?
            } else {
                func.call((typst,)).catch(&ctx).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Conversion failed: {}",
                        format_js_exception(e)
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

/// Convert Python dict to HashMap for custom_tex_macros
fn pydict_to_string_map(py_dict: &Bound<PyDict>) -> PyResult<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (key, value) in py_dict.iter() {
        let key_str: String = key.extract()?;
        let value_str: String = value.extract()?;
        map.insert(key_str, value_str);
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
///     non_strict: Allow non-strict parsing (default: None)
///     prefer_shorthands: Prefer shorthand notation (default: None)
///     keep_spaces: Preserve spaces in output (default: None)
///     frac_to_slash: Convert fractions to slash notation (default: None)
///     infty_to_oo: Convert infinity symbol to oo (default: None)
///     optimize: Optimize output (default: None)
///     custom_tex_macros: Custom TeX macro definitions (default: None)
///
/// Returns:
///     Converted Typst string
#[pyfunction]
#[pyo3(signature = (tex, *, non_strict=None, prefer_shorthands=None, keep_spaces=None, frac_to_slash=None, infty_to_oo=None, optimize=None, custom_tex_macros=None))]
#[allow(clippy::too_many_arguments)]
fn tex2typst(
    tex: String,
    non_strict: Option<bool>,
    prefer_shorthands: Option<bool>,
    keep_spaces: Option<bool>,
    frac_to_slash: Option<bool>,
    infty_to_oo: Option<bool>,
    optimize: Option<bool>,
    custom_tex_macros: Option<&Bound<PyDict>>,
) -> PyResult<String> {
    get_thread_converter()?;

    let mut options_map: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(val) = non_strict {
        options_map.insert("nonStrict".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(val) = prefer_shorthands {
        options_map.insert("preferShorthands".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(val) = keep_spaces {
        options_map.insert("keepSpaces".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(val) = frac_to_slash {
        options_map.insert("fracToSlash".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(val) = infty_to_oo {
        options_map.insert("inftyToOo".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(val) = optimize {
        options_map.insert("optimize".to_string(), serde_json::Value::Bool(val));
    }
    if let Some(macros) = custom_tex_macros {
        let macro_map = pydict_to_string_map(macros)?;
        options_map.insert(
            "customTexMacros".to_string(),
            serde_json::to_value(macro_map).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to serialize custom macros: {}",
                    e
                ))
            })?,
        );
    }

    let opts = if options_map.is_empty() {
        None
    } else {
        Some(options_map)
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
///     block_math_mode: Use block math mode (default: None)
///
/// Returns:
///     Converted LaTeX/TeX string
#[pyfunction]
#[pyo3(signature = (typst, *, block_math_mode=None))]
fn typst2tex(typst: String, block_math_mode: Option<bool>) -> PyResult<String> {
    get_thread_converter()?;

    let opts = if let Some(val) = block_math_mode {
        let mut options_map: HashMap<String, serde_json::Value> = HashMap::new();
        options_map.insert("blockMathMode".to_string(), serde_json::Value::Bool(val));
        Some(options_map)
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
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
