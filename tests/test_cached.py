"""Test LRU cached versions."""

import unittest
import time


class TestCachedAPI(unittest.TestCase):
    """Test Python-side LRU caching"""

    def test_cache_speeds_up_repeated_calls(self):
        """Verify that caching provides speedup for repeated conversions"""
        import tex2typst

        # Clear cache first
        tex2typst.clear_cache()

        test_input = r"\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"

        # First call (cache miss)
        start = time.perf_counter()
        for _ in range(100):
            result1 = tex2typst.tex2typst(test_input)
        time_uncached = time.perf_counter() - start

        # Second batch (all cache hits)
        start = time.perf_counter()
        for _ in range(100):
            result2 = tex2typst.tex2typst(test_input)
        time_cached = time.perf_counter() - start

        # Results should be identical
        self.assertEqual(result1, result2)

        # Cached should be much faster
        speedup = time_uncached / time_cached
        print("\n[Cache Performance]")
        print(f"  Uncached (100 calls): {time_uncached * 1000:.2f} ms")
        print(f"  Cached (100 calls): {time_cached * 1000:.2f} ms")
        print(f"  Speedup: {speedup:.1f}x")

        # Cache should provide significant speedup (at least 10x)
        self.assertGreater(speedup, 10)

    def test_cache_info(self):
        """Test cache_info returns statistics"""
        import tex2typst

        tex2typst.clear_cache()

        # Make some calls
        tex2typst.tex2typst(r"\alpha")
        tex2typst.tex2typst(r"\beta")
        tex2typst.tex2typst(r"\alpha")  # Cache hit

        info = tex2typst.cache_info()

        self.assertIn("tex2typst", info)
        self.assertEqual(info["tex2typst"].hits, 1)
        self.assertEqual(info["tex2typst"].misses, 2)
        self.assertEqual(info["tex2typst"].currsize, 2)

    def test_clear_cache(self):
        """Test cache clearing"""
        import tex2typst

        # Add some entries
        tex2typst.tex2typst(r"\alpha")
        tex2typst.typst2tex("beta")

        info_before = tex2typst.cache_info()
        self.assertGreater(info_before["tex2typst"].currsize, 0)

        # Clear cache
        tex2typst.clear_cache()

        info_after = tex2typst.cache_info()
        self.assertEqual(info_after["tex2typst"].currsize, 0)
        self.assertEqual(info_after["typst2tex"].currsize, 0)


if __name__ == "__main__":
    unittest.main(argv=["first-arg-is-ignored"], verbosity=2)
