import unittest
import tex2typst


class TestTex2Typst(unittest.TestCase):
    def setUp(self):
        self.converter = tex2typst.Tex2Typst()

    def test_basic_fraction(self):
        latex = "\\frac{1}{2}"
        result = self.converter.convert(latex)
        print(f"\n[Test Fraction] Input: {latex} -> Output: {result}")

        self.assertEqual(result.strip(), "1/2")

    def test_greek_letters(self):
        latex = "\\alpha + \\beta"
        result = self.converter.convert(latex)
        print(f"[Test Greek] Input: {latex} -> Output: {result}")

        self.assertEqual(result.strip(), "alpha + beta")

    def test_complex_formula(self):
        latex = "\\int_{-\\infty}^{\\infty} e^{-x^2} dx"
        result = self.converter.convert(latex)
        print(f"[Test Integral] Input: {latex} -> Output: {result}")

        self.assertEqual(result.strip(), "integral_(-infinity)^infinity e^(-x^2) d x")

    def test_empty_string(self):
        result = self.converter.convert("")
        self.assertEqual(result.strip(), "")

    def test_invalid_syntax_handling(self):
        latex = "\\notacommand{test}"
        try:
            result = self.converter.convert(latex)
            print(f"[Test Invalid] Input: {latex} -> Output: {result}")
            self.assertIsInstance(result, str)
        except Exception as e:
            self.fail(f"Converter crashed on invalid input: {e}")

    def test_thread_constraint(self):
        import threading

        def run_in_thread():
            try:
                c = pytex2typst.Tex2Typst()
                c.convert("x")
            except Exception as e:
                print(f"Thread error: {e}")

        t = threading.Thread(target=run_in_thread)
        t.start()
        t.join()


if __name__ == "__main__":
    unittest.main()
