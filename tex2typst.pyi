"""Type stubs for tex2typst module."""

from typing import overload

__version__: str

__all__ = ["tex2typst", "typst2tex", "clear_cache", "cache_info", "__version__"]

@overload
def tex2typst(
    tex: str,
    *,
    non_strict: bool | None = None,
    prefer_shorthands: bool | None = None,
    keep_spaces: bool | None = None,
    frac_to_slash: bool | None = None,
    infty_to_oo: bool | None = None,
    optimize: bool | None = None,
    custom_tex_macros: dict[str, str] | None = None,
) -> str:
    """
    Convert LaTeX/TeX math to Typst format.

    This function uses a thread-local lazy singleton pattern - the converter is
    initialized only on the first call within each thread, avoiding import-time overhead.

    Args:
        tex: LaTeX/TeX math string to convert
        non_strict: Allow non-strict parsing (default: library default)
        prefer_shorthands: Prefer shorthand notation (default: library default)
        keep_spaces: Preserve spaces in output (default: library default)
        frac_to_slash: Convert fractions to slash notation (default: library default)
        infty_to_oo: Convert infinity symbol to oo (default: library default)
        optimize: Optimize output (default: library default)
        custom_tex_macros: Custom TeX macro definitions as dict mapping macro names to expansions

    Returns:
        Converted Typst string

    Examples:
        >>> import tex2typst
        >>> tex2typst.tex2typst(r"\\frac{1}{2}")
        '1/2'
        >>> tex2typst.tex2typst(r"\\alpha + \\beta")
        'alpha + beta'
        >>> tex2typst.tex2typst(r"\\frac{1}{2}", frac_to_slash=False)
        'frac(1, 2)'
        >>> tex2typst.tex2typst(r"\\infty", infty_to_oo=True)
        'oo'
        >>> tex2typst.tex2typst(r"\\myop", custom_tex_macros={"\\\\myop": "\\\\operatorname{myop}"})
        'op("myop")'
    """
    ...

@overload
def tex2typst(
    tex: list[str],
    *,
    non_strict: bool | None = None,
    prefer_shorthands: bool | None = None,
    keep_spaces: bool | None = None,
    frac_to_slash: bool | None = None,
    infty_to_oo: bool | None = None,
    optimize: bool | None = None,
    custom_tex_macros: dict[str, str] | None = None,
) -> list[str]:
    """Convert multiple LaTeX/TeX strings to Typst format (with caching)."""
    ...

@overload
def typst2tex(typst: str, *, block_math_mode: bool | None = None) -> str:
    """
    Convert Typst math to LaTeX/TeX format.

    This function uses a thread-local lazy singleton pattern - the converter is
    initialized only on the first call within each thread, avoiding import-time overhead.

    Args:
        typst: Typst math string to convert
        block_math_mode: Use block math mode (default: library default)

    Returns:
        Converted LaTeX/TeX string

    Examples:
        >>> import tex2typst
        >>> tex2typst.typst2tex("1/2")
        '\\\\frac{1}{2}'
        >>> tex2typst.typst2tex("alpha + beta")
        '\\\\alpha + \\\\beta'
        >>> tex2typst.typst2tex("x", block_math_mode=False)
        'x'
    """
    ...

@overload
def typst2tex(typst: list[str], *, block_math_mode: bool | None = None) -> list[str]:
    """Convert multiple Typst strings to LaTeX/TeX format (with caching)."""
    ...

def clear_cache() -> None:
    """Clear all cached conversion results."""
    ...

def cache_info() -> dict[str, object]:
    """Get cache statistics for tex2typst and typst2tex."""
    ...
