"""
tex2typst: Convert between LaTeX/TeX and Typst math notation.

This module provides conversion functions with built-in LRU caching for improved
performance on repeated conversions.

Usage:
    >>> import tex2typst
    >>> result = tex2typst.tex2typst(r"\\frac{1}{2}")
    >>> result = tex2typst.tex2typst(r"\\frac{1}{2}")  # Cache hit!
    >>> tex2typst.cache_info()  # View cache statistics
    >>> tex2typst.clear_cache()  # Clear cache
"""

from functools import lru_cache
from typing import Optional, Dict, Union, List, overload
from . import _tex2typst_core

__version__ = _tex2typst_core.__version__


def _make_hashable(d: Optional[Dict[str, str]]) -> Optional[tuple]:
    """Convert dict to hashable tuple for caching."""
    return tuple(sorted(d.items())) if d is not None else None


@lru_cache(maxsize=1024)
def _tex2typst_cached(
    tex: str,
    non_strict: Optional[bool],
    prefer_shorthands: Optional[bool],
    keep_spaces: Optional[bool],
    frac_to_slash: Optional[bool],
    infty_to_oo: Optional[bool],
    optimize: Optional[bool],
    custom_tex_macros: Optional[tuple],
) -> str:
    """Internal cached function with hashable parameters."""
    macros = dict(custom_tex_macros) if custom_tex_macros else None
    return _tex2typst_core.tex2typst(
        tex,
        non_strict=non_strict,
        prefer_shorthands=prefer_shorthands,
        keep_spaces=keep_spaces,
        frac_to_slash=frac_to_slash,
        infty_to_oo=infty_to_oo,
        optimize=optimize,
        custom_tex_macros=macros,
    )


@overload
def tex2typst(
    tex: str,
    *,
    non_strict: Optional[bool] = None,
    prefer_shorthands: Optional[bool] = None,
    keep_spaces: Optional[bool] = None,
    frac_to_slash: Optional[bool] = None,
    infty_to_oo: Optional[bool] = None,
    optimize: Optional[bool] = None,
    custom_tex_macros: Optional[Dict[str, str]] = None,
) -> str: ...


@overload
def tex2typst(
    tex: List[str],
    *,
    non_strict: Optional[bool] = None,
    prefer_shorthands: Optional[bool] = None,
    keep_spaces: Optional[bool] = None,
    frac_to_slash: Optional[bool] = None,
    infty_to_oo: Optional[bool] = None,
    optimize: Optional[bool] = None,
    custom_tex_macros: Optional[Dict[str, str]] = None,
) -> List[str]: ...


def tex2typst(
    tex: Union[str, List[str]],
    *,
    non_strict: Optional[bool] = None,
    prefer_shorthands: Optional[bool] = None,
    keep_spaces: Optional[bool] = None,
    frac_to_slash: Optional[bool] = None,
    infty_to_oo: Optional[bool] = None,
    optimize: Optional[bool] = None,
    custom_tex_macros: Optional[Dict[str, str]] = None,
) -> Union[str, List[str]]:
    """
    Convert LaTeX/TeX to Typst format (with LRU caching).

    Intelligently handles both single strings and lists of strings.
    Results are cached automatically for improved performance on repeated conversions.

    Args:
        tex: LaTeX/TeX math string or list of strings to convert
        non_strict: Allow non-strict parsing
        prefer_shorthands: Prefer shorthand notation
        keep_spaces: Preserve spaces in output
        frac_to_slash: Convert fractions to slash notation
        infty_to_oo: Convert infinity symbol to oo
        optimize: Optimize output
        custom_tex_macros: Custom TeX macro definitions

    Returns:
        Converted Typst string or list of strings (matches input type)

    Examples:
        >>> tex2typst(r"\\frac{1}{2}")
        '1/2'
        >>> tex2typst([r"\\alpha", r"\\beta"])
        ['alpha', 'beta']
    """
    if isinstance(tex, str):
        # Single string: use cached function
        macros_tuple = _make_hashable(custom_tex_macros)
        return _tex2typst_cached(
            tex,
            non_strict,
            prefer_shorthands,
            keep_spaces,
            frac_to_slash,
            infty_to_oo,
            optimize,
            macros_tuple,
        )
    elif isinstance(tex, list):
        # List: use batch processing API for better performance
        # Batch API processes all items in one Rust/JS context entry, reducing overhead
        return _tex2typst_core.tex2typst_batch(
            tex,
            non_strict=non_strict,
            prefer_shorthands=prefer_shorthands,
            keep_spaces=keep_spaces,
            frac_to_slash=frac_to_slash,
            infty_to_oo=infty_to_oo,
            optimize=optimize,
            custom_tex_macros=custom_tex_macros,
        )
    else:
        raise TypeError(f"Expected str or list, got {type(tex).__name__}")


@lru_cache(maxsize=1024)
def _typst2tex_cached(
    typst: str,
    block_math_mode: Optional[bool],
) -> str:
    """Internal cached function."""
    return _tex2typst_core.typst2tex(typst, block_math_mode=block_math_mode)


@overload
def typst2tex(
    typst: str,
    *,
    block_math_mode: Optional[bool] = None,
) -> str: ...


@overload
def typst2tex(
    typst: List[str],
    *,
    block_math_mode: Optional[bool] = None,
) -> List[str]: ...


def typst2tex(
    typst: Union[str, List[str]],
    *,
    block_math_mode: Optional[bool] = None,
) -> Union[str, List[str]]:
    """
    Convert Typst to LaTeX/TeX format (with LRU caching).

    Intelligently handles both single strings and lists of strings.
    Results are cached automatically for improved performance on repeated conversions.

    Args:
        typst: Typst math string or list of strings to convert
        block_math_mode: Use block math mode

    Returns:
        Converted LaTeX/TeX string or list of strings (matches input type)

    Examples:
        >>> typst2tex("1/2")
        '\\\\frac{1}{2}'
        >>> typst2tex(["alpha", "beta"])
        ['\\\\alpha', '\\\\beta']
    """
    if isinstance(typst, str):
        return _typst2tex_cached(typst, block_math_mode)
    elif isinstance(typst, list):
        # List: use batch processing API internally for better performance
        return _tex2typst_core.typst2tex_batch(
            typst,
            block_math_mode=block_math_mode,
        )
    else:
        raise TypeError(f"Expected str or list, got {type(typst).__name__}")


def clear_cache() -> None:
    """
    Clear all cached conversion results.

    Example:
        >>> clear_cache()
    """
    _tex2typst_cached.cache_clear()
    _typst2tex_cached.cache_clear()


def cache_info() -> Dict[str, object]:
    """
    Get cache statistics.

    Returns:
        Dictionary with cache info for tex2typst and typst2tex

    Example:
        >>> info = cache_info()
        >>> print(f"tex2typst hits: {info['tex2typst'].hits}")
        >>> print(f"tex2typst misses: {info['tex2typst'].misses}")
        >>> print(f"Cache size: {info['tex2typst'].currsize}/{info['tex2typst'].maxsize}")
    """
    return {
        "tex2typst": _tex2typst_cached.cache_info(),
        "typst2tex": _typst2tex_cached.cache_info(),
    }


__all__ = [
    "tex2typst",
    "typst2tex",
    "clear_cache",
    "cache_info",
    "__version__",
]
