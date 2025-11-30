# tex2typst

A Python wrapper for the [tex2typst](https://github.com/qwinsi/tex2typst) JavaScript library, providing seamless conversion between LaTeX/TeX math and Typst format.

## Features

- **Bidirectional Conversion**: Convert from LaTeX to Typst and vice versa
- **Fast Performance**: Built with Rust using PyO3 for native performance
- **Thread-Safe**: Uses thread-local lazy singletons for efficient multi-threaded usage
- **Flexible Options**: Customize conversion behavior with various options
- **Custom Macros**: Support for custom TeX macro definitions

## Installation

```bash
pip install tex2typst
```

## Quick Start

### LaTeX to Typst

```python
import tex2typst

# Basic conversion
result = tex2typst.tex2typst(r"\frac{1}{2}")
print(result)  # Output: 1/2

# Greek letters
result = tex2typst.tex2typst(r"\alpha + \beta")
print(result)  # Output: alpha + beta

# Complex formulas
result = tex2typst.tex2typst(r"\int_{-\infty}^{\infty} e^{-x^2} dx")
print(result)  # Output: integral_(-infinity)^infinity e^(-x^2) d x
```

### Typst to LaTeX

```python
import tex2typst

# Basic conversion
result = tex2typst.typst2tex("1/2")
print(result)  # Output: \frac{1}{2}

# Greek letters
result = tex2typst.typst2tex("alpha + beta")
print(result)  # Output: \alpha + \beta
```

## Advanced Usage

### Conversion Options for tex2typst

```python
import tex2typst

# Disable fraction to slash conversion
result = tex2typst.tex2typst(r"\frac{1}{2}", frac_to_slash=False)
print(result)  # Output: frac(1, 2)

# Convert infinity to oo
result = tex2typst.tex2typst(r"\infty", infty_to_oo=True)
print(result)  # Output: oo

# Custom TeX macros
result = tex2typst.tex2typst(
    r"\myop x = \sgn(y)",
    custom_tex_macros={
        r"\myop": r"\operatorname{myop}",
        r"\sgn": r"\operatorname{sgn}",
    }
)
print(result)  # Output: op("myop") x = op("sgn")(y)

# Multiple options
result = tex2typst.tex2typst(
    r"\frac{1}{\infty}",
    frac_to_slash=False,
    infty_to_oo=True
)
print(result)  # Output: frac(1, oo)
```

### Available Options for tex2typst

- `non_strict` (bool): Allow non-strict parsing
- `prefer_shorthands` (bool): Prefer shorthand notation
- `keep_spaces` (bool): Preserve spaces in output
- `frac_to_slash` (bool): Convert fractions to slash notation (default: True)
- `infty_to_oo` (bool): Convert infinity symbol to oo
- `optimize` (bool): Optimize output
- `custom_tex_macros` (dict[str, str]): Custom TeX macro definitions

### Conversion Options for typst2tex

```python
import tex2typst

# Use block math mode
result = tex2typst.typst2tex("x", block_math_mode=True)

# Inline math mode
result = tex2typst.typst2tex("x", block_math_mode=False)
```

### Available Options for typst2tex

- `block_math_mode` (bool): Use block math mode

## How It Works

This library wraps the JavaScript `tex2typst` library using:
- **Rust**: Core binding logic using PyO3
- **rquickjs**: JavaScript runtime for executing the tex2typst library
- **maturin**: Build system for Python extension modules

The converter uses a thread-local lazy singleton pattern, meaning:
- The JavaScript runtime is initialized only once per thread
- First call has a slight overhead; subsequent calls are fast
- Thread-safe for use in multi-threaded applications
- No import-time overhead

## Development

### Prerequisites

- Rust (latest stable)
- Python 3.8+
- Node.js and yarn (for building JavaScript bundle)
- Just (command runner)

### Setup

```bash
# Clone the repository
git clone https://github.com/AndPuQing/tex2typst-py.git
cd tex2typst-py

# Install dependencies
yarn install

# Build JavaScript bundle and install Python package
just dev
```

### Building

```bash
# Build JavaScript bundle
just dist

# Build Python wheel
just build

# Install in development mode
just dev
```

### Running Tests

```bash
# Run tests
pytest tests/

# Run specific test file
pytest tests/test_pytex2typst.py -v
```

### Pre-commit Hooks

This project uses pre-commit hooks for code quality:

```bash
# Install pre-commit hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## Project Structure

```
.
├── src/
│   └── lib.rs           # Rust bindings (PyO3)
├── js/
│   └── tex2typst.bundle.js  # Bundled JavaScript
├── tests/
│   ├── test_pytex2typst.py  # Unit tests
│   └── test_benchmark.py     # Benchmarks
├── entry.js             # JavaScript entry point
├── tex2typst.pyi        # Python type stubs
├── Cargo.toml           # Rust dependencies
├── pyproject.toml       # Python project config
└── justfile             # Build commands
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License.

## Acknowledgments

- [tex2typst](https://github.com/qwinsi/tex2typst) - The underlying tex2typst JavaScript library
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [rquickjs](https://github.com/DelSkayn/rquickjs) - QuickJS bindings for Rust

## Related Projects

- [mitex](https://github.com/mitex-rs/mitex) - LaTeX support for Typst
- [typst](https://typst.app/) - A modern markup-based typesetting system

## See Also

- [Typst Documentation](https://typst.app/docs/)
- [LaTeX Math Symbols](https://www.ctan.org/pkg/comprehensive)
