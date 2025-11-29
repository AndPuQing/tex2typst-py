use pyo3::prelude::*;
use rquickjs::{Context, Function, Runtime};

const JS_CODE: &str = include_str!("../dist/tex2typst.bundle.js");

#[pyclass(unsendable)]
struct Tex2Typst {
    ctx: Context,
}

#[pymethods]
impl Tex2Typst {
    #[new]
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

        Ok(Tex2Typst { ctx })
    }

    fn convert(&self, latex: String) -> PyResult<String> {
        self.ctx.with(|ctx| {
            let globals = ctx.globals();

            let func: Function = globals.get("convert").map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    "Global function 'convert' not found.",
                )
            })?;

            let result: String = func.call((latex,)).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Conversion failed: {}", e))
            })?;

            Ok(result)
        })
    }
}

#[pymodule]
fn tex2typst(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Tex2Typst>()?;
    Ok(())
}
