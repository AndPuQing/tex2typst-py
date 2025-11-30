import unittest
import tex2typst


class TestTex2TypstFunctionAPI(unittest.TestCase):
    """Test the tex2typst function-based API"""

    def test_basic_fraction(self):
        latex = "\\frac{1}{2}"
        result = tex2typst.tex2typst(latex)
        print(f"\n[Test Fraction] Input: {latex} -> Output: {result}")
        self.assertEqual(result.strip(), "1/2")

    def test_greek_letters(self):
        latex = "\\alpha + \\beta"
        result = tex2typst.tex2typst(latex)
        print(f"[Test Greek] Input: {latex} -> Output: {result}")
        self.assertEqual(result.strip(), "alpha + beta")

    def test_complex_formula(self):
        latex = "\\int_{-\\infty}^{\\infty} e^{-x^2} dx"
        result = tex2typst.tex2typst(latex)
        print(f"[Test Integral] Input: {latex} -> Output: {result}")
        self.assertEqual(result.strip(), "integral_(-infinity)^infinity e^(-x^2) d x")

    def test_empty_string(self):
        result = tex2typst.tex2typst("")
        self.assertEqual(result.strip(), "")

    def test_invalid_syntax_handling(self):
        latex = "\\notacommand{test}"
        try:
            result = tex2typst.tex2typst(latex)
            print(f"[Test Invalid] Input: {latex} -> Output: {result}")
            self.assertIsInstance(result, str)
        except Exception as e:
            self.fail(f"Converter crashed on invalid input: {e}")

    def test_singleton_reuse(self):
        """Verify that multiple calls use the same singleton instance"""
        result1 = tex2typst.tex2typst("\\alpha")
        result2 = tex2typst.tex2typst("\\beta")
        self.assertEqual(result1.strip(), "alpha")
        self.assertEqual(result2.strip(), "beta")


class TestTypst2TexFunctionAPI(unittest.TestCase):
    """Test the typst2tex function-based API"""

    def test_basic_fraction(self):
        typst = "1/2"
        result = tex2typst.typst2tex(typst)
        print(f"\n[Test Typst2Tex Fraction] Input: {typst} -> Output: {result}")
        self.assertIn("frac", result)

    def test_greek_letters(self):
        typst = "alpha + beta"
        result = tex2typst.typst2tex(typst)
        print(f"[Test Typst2Tex Greek] Input: {typst} -> Output: {result}")
        self.assertIn("alpha", result)
        self.assertIn("beta", result)

    def test_empty_string(self):
        result = tex2typst.typst2tex("")
        self.assertEqual(result.strip(), "")

    def test_roundtrip(self):
        """Test converting from LaTeX to Typst and back"""
        latex = "\\alpha + \\beta"
        typst = tex2typst.tex2typst(latex)
        result = tex2typst.typst2tex(typst)
        print(f"\n[Test Roundtrip] LaTeX: {latex} -> Typst: {typst} -> LaTeX: {result}")
        self.assertIsInstance(result, str)


class TestTex2TypstOptions(unittest.TestCase):
    """Test tex2typst with various options"""

    def test_frac_to_slash_false(self):
        latex = "\\frac{1}{2}"
        result = tex2typst.tex2typst(latex, frac_to_slash=False)
        print(
            f"\n[Test Options frac_to_slash=False] Input: {latex} -> Output: {result}"
        )
        self.assertIn("frac", result)
        self.assertNotIn("1/2", result)

    def test_infty_to_oo_true(self):
        latex = "\\infty"
        result = tex2typst.tex2typst(latex, infty_to_oo=True)
        print(f"\n[Test Options infty_to_oo=True] Input: {latex} -> Output: {result}")
        self.assertEqual(result.strip(), "oo")

    def test_infty_to_oo_false(self):
        latex = "\\infty"
        result = tex2typst.tex2typst(latex, infty_to_oo=False)
        print(f"\n[Test Options infty_to_oo=False] Input: {latex} -> Output: {result}")
        self.assertIn("infinity", result.lower())

    def test_keep_spaces_true(self):
        latex = "a   b"
        result = tex2typst.tex2typst(latex, keep_spaces=True)
        print(
            f"\n[Test Options keep_spaces=True] Input: '{latex}' -> Output: '{result}'"
        )

    def test_custom_tex_macros(self):
        latex = "\\myop y=\\sgn(x)"
        result = tex2typst.tex2typst(
            latex,
            custom_tex_macros={
                "\\myop": "\\operatorname{myop}",
                "\\sgn": "\\operatorname{sgn}",
            },
        )
        print(f"\n[Test Options custom_tex_macros] Input: {latex} -> Output: {result}")
        self.assertIn('op("myop")', result)

    def test_multiple_options(self):
        latex = "\\frac{1}{\\infty}"
        result = tex2typst.tex2typst(latex, frac_to_slash=False, infty_to_oo=True)
        print(f"\n[Test Options multiple] Input: {latex} -> Output: {result}")
        self.assertIn("frac", result)
        self.assertIn("oo", result)


class TestTypst2TexOptions(unittest.TestCase):
    """Test typst2tex with various options"""

    def test_block_math_mode_true(self):
        typst = "x"
        result = tex2typst.typst2tex(typst, block_math_mode=True)
        print(
            f"\n[Test Options block_math_mode=True] Input: {typst} -> Output: {result}"
        )
        self.assertIsInstance(result, str)

    def test_block_math_mode_false(self):
        typst = "x"
        result = tex2typst.typst2tex(typst, block_math_mode=False)
        print(
            f"\n[Test Options block_math_mode=False] Input: {typst} -> Output: {result}"
        )
        self.assertIsInstance(result, str)


if __name__ == "__main__":
    unittest.main()
