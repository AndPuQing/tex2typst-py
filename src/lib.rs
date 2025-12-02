use pyo3::prelude::*;
use pyo3::types::PyDict;
use rquickjs::{CatchResultExt, CaughtError, Context, Function, Object, Runtime};
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
/// The JavaScript code is loaded once per thread via lazy singleton pattern
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

        // Evaluate JavaScript code once during initialization
        // This is already optimized via the thread-local lazy singleton pattern
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
        options: Option<&HashMap<String, serde_json::Value>>,
    ) -> PyResult<String> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();
            let func: Function = globals.get("tex2typst").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'tex2typst' not found.",
                )
            })?;

            let result: String = if let Some(opts) = options {
                // Direct object construction (OPTIMIZATION: avoid full JSON serialization)
                let js_options = Object::new(ctx.clone()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create JS object: {}",
                        e
                    ))
                })?;

                // Set properties directly without JSON serialization
                for (key, value) in opts.iter() {
                    match value {
                        serde_json::Value::Bool(b) => {
                            js_options.set(key.as_str(), *b).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set bool property: {}",
                                    e
                                ))
                            })?;
                        }
                        serde_json::Value::String(s) => {
                            js_options.set(key.as_str(), s.as_str()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set string property: {}",
                                    e
                                ))
                            })?;
                        }
                        serde_json::Value::Object(obj) => {
                            let nested_obj = Object::new(ctx.clone()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to create nested object: {}",
                                    e
                                ))
                            })?;
                            for (k, v) in obj.iter() {
                                if let serde_json::Value::String(s) = v {
                                    nested_obj.set(k.as_str(), s.as_str()).map_err(|e| {
                                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                            "Failed to set nested property: {}",
                                            e
                                        ))
                                    })?;
                                }
                            }
                            js_options.set(key.as_str(), nested_obj).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set object property: {}",
                                    e
                                ))
                            })?;
                        }
                        _ => {
                            // Fallback to JSON for other types
                            let js_val = ctx.json_parse(value.to_string()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                    "Options parse failed: {}",
                                    e
                                ))
                            })?;
                            js_options.set(key.as_str(), js_val).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set property: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }

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

    /// Batch process multiple tex strings - reduces Rust<->JS boundary crossings
    fn tex2typst_batch(
        &self,
        tex_list: &[String],
        options: Option<&HashMap<String, serde_json::Value>>,
    ) -> PyResult<Vec<String>> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();
            let func: Function = globals.get("tex2typst").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'tex2typst' not found.",
                )
            })?;

            let mut results = Vec::with_capacity(tex_list.len());

            // Pre-create options object once if needed (shared across all conversions)
            let js_options_obj = if let Some(opts) = options {
                let js_options = Object::new(ctx.clone()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create JS object: {}",
                        e
                    ))
                })?;

                for (key, value) in opts.iter() {
                    match value {
                        serde_json::Value::Bool(b) => {
                            js_options.set(key.as_str(), *b).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set bool property: {}",
                                    e
                                ))
                            })?;
                        }
                        serde_json::Value::String(s) => {
                            js_options.set(key.as_str(), s.as_str()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set string property: {}",
                                    e
                                ))
                            })?;
                        }
                        serde_json::Value::Object(obj) => {
                            let nested_obj = Object::new(ctx.clone()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to create nested object: {}",
                                    e
                                ))
                            })?;
                            for (k, v) in obj.iter() {
                                if let serde_json::Value::String(s) = v {
                                    nested_obj.set(k.as_str(), s.as_str()).map_err(|e| {
                                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                            "Failed to set nested property: {}",
                                            e
                                        ))
                                    })?;
                                }
                            }
                            js_options.set(key.as_str(), nested_obj).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set object property: {}",
                                    e
                                ))
                            })?;
                        }
                        _ => {
                            let js_val = ctx.json_parse(value.to_string()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                    "Options parse failed: {}",
                                    e
                                ))
                            })?;
                            js_options.set(key.as_str(), js_val).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set property: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
                Some(js_options)
            } else {
                None
            };

            // Process all items in a single context entry
            for tex in tex_list {
                let result: String = if let Some(ref js_opts) = js_options_obj {
                    func.call((tex.as_str(), js_opts.clone()))
                        .catch(&ctx)
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                "Conversion failed for '{}': {}",
                                tex,
                                format_js_exception(e)
                            ))
                        })?
                } else {
                    func.call((tex.as_str(),)).catch(&ctx).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Conversion failed for '{}': {}",
                            tex,
                            format_js_exception(e)
                        ))
                    })?
                };
                results.push(result);
            }

            Ok(results)
        })
    }

    fn typst2tex(
        &self,
        typst: &str,
        options: Option<&HashMap<String, serde_json::Value>>,
    ) -> PyResult<String> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();
            let func: Function = globals.get("typst2tex").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'typst2tex' not found.",
                )
            })?;

            let result: String = if let Some(opts) = options {
                // Direct object construction (OPTIMIZATION: avoid full JSON serialization)
                let js_options = Object::new(ctx.clone()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create JS object: {}",
                        e
                    ))
                })?;

                // Set properties directly without JSON serialization
                for (key, value) in opts.iter() {
                    match value {
                        serde_json::Value::Bool(b) => {
                            js_options.set(key.as_str(), *b).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set bool property: {}",
                                    e
                                ))
                            })?;
                        }
                        _ => {
                            // Fallback to JSON for other types
                            let js_val = ctx.json_parse(value.to_string()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                    "Options parse failed: {}",
                                    e
                                ))
                            })?;
                            js_options.set(key.as_str(), js_val).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set property: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }

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

    /// Batch process multiple typst strings - reduces Rust<->JS boundary crossings
    fn typst2tex_batch(
        &self,
        typst_list: &[String],
        options: Option<&HashMap<String, serde_json::Value>>,
    ) -> PyResult<Vec<String>> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();
            let func: Function = globals.get("typst2tex").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'typst2tex' not found.",
                )
            })?;

            let mut results = Vec::with_capacity(typst_list.len());

            // Pre-create options object once if needed (shared across all conversions)
            let js_options_obj = if let Some(opts) = options {
                let js_options = Object::new(ctx.clone()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create JS object: {}",
                        e
                    ))
                })?;

                for (key, value) in opts.iter() {
                    match value {
                        serde_json::Value::Bool(b) => {
                            js_options.set(key.as_str(), *b).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set bool property: {}",
                                    e
                                ))
                            })?;
                        }
                        _ => {
                            let js_val = ctx.json_parse(value.to_string()).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                    "Options parse failed: {}",
                                    e
                                ))
                            })?;
                            js_options.set(key.as_str(), js_val).map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                                    "Failed to set property: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
                Some(js_options)
            } else {
                None
            };

            // Process all items in a single context entry
            for typst in typst_list {
                let result: String = if let Some(ref js_opts) = js_options_obj {
                    func.call((typst.as_str(), js_opts.clone()))
                        .catch(&ctx)
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                                "Conversion failed for '{}': {}",
                                typst,
                                format_js_exception(e)
                            ))
                        })?
                } else {
                    func.call((typst.as_str(),)).catch(&ctx).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "Conversion failed for '{}': {}",
                            typst,
                            format_js_exception(e)
                        ))
                    })?
                };
                results.push(result);
            }

            Ok(results)
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

    // Pre-allocate with capacity for 7 possible options (OPTIMIZATION #4)
    let mut options_map: HashMap<String, serde_json::Value> = HashMap::with_capacity(7);

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

    THREAD_CONVERTER.with(|converter| {
        converter
            .borrow()
            .as_ref()
            .unwrap()
            .tex2typst(&tex, opts.as_ref())
    })
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

    THREAD_CONVERTER.with(|converter| {
        converter
            .borrow()
            .as_ref()
            .unwrap()
            .typst2tex(&typst, opts.as_ref())
    })
}

/// Batch convert multiple LaTeX/TeX strings to Typst format (internal batch API).
///
/// This function is used internally by the Python wrapper to optimize list processing.
/// It processes all conversions in a single Rust/JS context entry, reducing overhead.
#[pyfunction]
#[pyo3(signature = (tex_list, *, non_strict=None, prefer_shorthands=None, keep_spaces=None, frac_to_slash=None, infty_to_oo=None, optimize=None, custom_tex_macros=None))]
#[allow(clippy::too_many_arguments)]
fn tex2typst_batch(
    tex_list: Vec<String>,
    non_strict: Option<bool>,
    prefer_shorthands: Option<bool>,
    keep_spaces: Option<bool>,
    frac_to_slash: Option<bool>,
    infty_to_oo: Option<bool>,
    optimize: Option<bool>,
    custom_tex_macros: Option<&Bound<PyDict>>,
) -> PyResult<Vec<String>> {
    get_thread_converter()?;

    let mut options_map: HashMap<String, serde_json::Value> = HashMap::with_capacity(7);

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

    THREAD_CONVERTER.with(|converter| {
        converter
            .borrow()
            .as_ref()
            .unwrap()
            .tex2typst_batch(&tex_list, opts.as_ref())
    })
}

/// Batch convert multiple Typst strings to LaTeX/TeX format (internal batch API).
///
/// This function is used internally by the Python wrapper to optimize list processing.
/// It processes all conversions in a single Rust/JS context entry, reducing overhead.
#[pyfunction]
#[pyo3(signature = (typst_list, *, block_math_mode=None))]
fn typst2tex_batch(
    typst_list: Vec<String>,
    block_math_mode: Option<bool>,
) -> PyResult<Vec<String>> {
    get_thread_converter()?;

    let opts = if let Some(val) = block_math_mode {
        let mut options_map: HashMap<String, serde_json::Value> = HashMap::new();
        options_map.insert("blockMathMode".to_string(), serde_json::Value::Bool(val));
        Some(options_map)
    } else {
        None
    };

    THREAD_CONVERTER.with(|converter| {
        converter
            .borrow()
            .as_ref()
            .unwrap()
            .typst2tex_batch(&typst_list, opts.as_ref())
    })
}

#[pymodule]
#[pyo3(name = "_tex2typst_core")]
fn tex2typst_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(tex2typst, m)?)?;
    m.add_function(wrap_pyfunction!(typst2tex, m)?)?;
    m.add_function(wrap_pyfunction!(tex2typst_batch, m)?)?;
    m.add_function(wrap_pyfunction!(typst2tex_batch, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
