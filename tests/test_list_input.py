"""Test list input support."""

import unittest
import tex2typst


class TestListInput(unittest.TestCase):
    """Test list input functionality"""

    def test_tex2typst_with_list(self):
        """Test tex2typst with list input"""
        inputs = [r"\alpha", r"\beta", r"\gamma"]
        results = tex2typst.tex2typst(inputs)

        self.assertIsInstance(results, list)
        self.assertEqual(len(results), 3)
        self.assertEqual(results[0], "alpha")
        self.assertEqual(results[1], "beta")
        self.assertEqual(results[2], "gamma")

    def test_tex2typst_with_string(self):
        """Test tex2typst with string input still works"""
        result = tex2typst.tex2typst(r"\alpha")

        self.assertIsInstance(result, str)
        self.assertEqual(result, "alpha")

    def test_tex2typst_list_with_options(self):
        """Test list conversion with options"""
        inputs = [r"\frac{1}{2}", r"\frac{3}{4}"]
        results = tex2typst.tex2typst(inputs, frac_to_slash=False)

        self.assertEqual(len(results), 2)
        self.assertIn("frac", results[0])
        self.assertIn("frac", results[1])

    def test_typst2tex_with_list(self):
        """Test typst2tex with list input"""
        inputs = ["alpha", "beta", "gamma"]
        results = tex2typst.typst2tex(inputs)

        self.assertIsInstance(results, list)
        self.assertEqual(len(results), 3)
        self.assertEqual(results[0], r"\alpha")
        self.assertEqual(results[1], r"\beta")
        self.assertEqual(results[2], r"\gamma")

    def test_typst2tex_with_string(self):
        """Test typst2tex with string input still works"""
        result = tex2typst.typst2tex("alpha")

        self.assertIsInstance(result, str)
        self.assertEqual(result, r"\alpha")

    def test_empty_list(self):
        """Test with empty list"""
        results = tex2typst.tex2typst([])
        self.assertEqual(results, [])

        results = tex2typst.typst2tex([])
        self.assertEqual(results, [])

    def test_list_caching(self):
        """Test that single strings use cache, while lists use batch processing"""
        tex2typst.clear_cache()

        # Single string conversions use cache
        tex2typst.tex2typst(r"\alpha")
        tex2typst.tex2typst(r"\beta")
        tex2typst.tex2typst(r"\alpha")  # Should hit cache

        info = tex2typst.cache_info()
        self.assertGreater(info["tex2typst"].hits, 0, "Single strings should use cache")

        # List conversions use batch processing (bypasses cache for performance)
        tex2typst.clear_cache()
        tex2typst.tex2typst([r"\alpha", r"\beta"])
        tex2typst.tex2typst([r"\alpha", r"\gamma"])

        info = tex2typst.cache_info()
        # Batch API doesn't go through Python-level cache
        self.assertEqual(info["tex2typst"].hits, 0, "Lists use batch API, not cache")

    def test_invalid_type(self):
        """Test that invalid types raise TypeError"""
        with self.assertRaises(TypeError):
            tex2typst.tex2typst(123)  # type: ignore

        with self.assertRaises(TypeError):
            tex2typst.typst2tex({"key": "value"})  # type: ignore


if __name__ == "__main__":
    unittest.main(argv=["first-arg-is-ignored"], verbosity=2)
