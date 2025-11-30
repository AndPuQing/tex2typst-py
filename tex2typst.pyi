from typing import TypedDict

class Tex2TypstOptions(TypedDict, total=False):
    """
    Options for LaTeX to Typst conversion.

    Attributes:
        nonStrict: Allow non-strict parsing (default: True)
        preferShorthands: Prefer shorthand notation (default: True)
        keepSpaces: Preserve spaces in output (default: False)
        fracToSlash: Convert fractions to slash notation (default: True)
        inftyToOo: Convert infinity symbol to oo (default: False)
        optimize: Optimize output (default: True)
        customTexMacros: Custom TeX macro definitions as a dict mapping macro names to their expansions
    """
    nonStrict: bool
    preferShorthands: bool
    keepSpaces: bool
    fracToSlash: bool
    inftyToOo: bool
    optimize: bool
    customTexMacros: dict[str, str]

class Typst2TexOptions(TypedDict, total=False):
    """
    Options for Typst to LaTeX conversion.

    Attributes:
        blockMathMode: Use block math mode (default: True)
    """
    blockMathMode: bool

def tex2typst(tex: str, options: Tex2TypstOptions | None = None) -> str:
    """
    Convert LaTeX/TeX math to Typst format.

    This function uses a thread-local lazy singleton pattern - the converter is
    initialized only on the first call within each thread, avoiding import-time overhead.

    Args:
        tex: LaTeX/TeX math string to convert
        options: Optional conversion options

    Returns:
        Converted Typst string

    Examples:
        >>> import tex2typst
        >>> tex2typst.tex2typst(r"\\frac{1}{2}")
        '1/2'
        >>> tex2typst.tex2typst(r"\\alpha + \\beta")
        'alpha + beta'
        >>> tex2typst.tex2typst(r"\\frac{1}{2}", {"fracToSlash": False})
        'frac(1, 2)'
        >>> tex2typst.tex2typst(r"\\infty", {"inftyToOo": True})
        'oo'
    """
    ...

def typst2tex(typst: str, options: Typst2TexOptions | None = None) -> str:
    """
    Convert Typst math to LaTeX/TeX format.

    This function uses a thread-local lazy singleton pattern - the converter is
    initialized only on the first call within each thread, avoiding import-time overhead.

    Args:
        typst: Typst math string to convert
        options: Optional conversion options

    Returns:
        Converted LaTeX/TeX string

    Examples:
        >>> import tex2typst
        >>> tex2typst.typst2tex("1/2")
        '\\\\frac{1}{2}'
        >>> tex2typst.typst2tex("alpha + beta")
        '\\\\alpha + \\\\beta'
        >>> tex2typst.typst2tex("x", {"blockMathMode": False})
        'x'
    """
    ...
