import unittest
import time
import tex2typst


class TestPerformanceFunctionAPI(unittest.TestCase):
    """Benchmark using the function-based API with lazy singleton"""

    @classmethod
    def setUpClass(cls):
        print("\n[Setup] Testing lazy initialization with function API...")
        # First call initializes the singleton
        start_t = time.perf_counter()
        tex2typst.tex2typst("\\alpha")
        init_time = (time.perf_counter() - start_t) * 1000
        print(f"[Setup] First call (with init): {init_time:.2f} ms")

        # Second call should be fast (singleton already initialized)
        start_t = time.perf_counter()
        tex2typst.tex2typst("\\alpha")
        subsequent_time = (time.perf_counter() - start_t) * 1000
        print(f"[Setup] Second call (no init): {subsequent_time:.2f} ms")

    def benchmark(self, name, latex_input, iterations=10000):
        # Warmup
        for _ in range(100):
            tex2typst.tex2typst(latex_input)

        start_time = time.perf_counter()

        for _ in range(iterations):
            tex2typst.tex2typst(latex_input)

        end_time = time.perf_counter()

        total_time = end_time - start_time
        avg_latency_ms = (total_time / iterations) * 1000
        throughput_qps = iterations / total_time

        print(f"\n--- Benchmark (Function API): {name} ---")
        print(f"Iterations : {iterations}")
        print(f"Total Time : {total_time:.4f} s")
        print(f"Latency    : {avg_latency_ms:.4f} ms/op")
        print(f"Throughput : {throughput_qps:.0f} ops/sec")

        self.assertGreater(throughput_qps, 10, "Throughput is surprisingly low!")

    def test_perf_simple(self):
        latex = "\\alpha"
        self.benchmark("Simple Token", latex, iterations=500)

    def test_perf_medium(self):
        latex = "\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}"
        self.benchmark("Quadratic Formula", latex, iterations=200)

    def test_perf_complex(self):
        latex = r"\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \sqrt{\\pi} \\quad \\text{where } x \\in \\mathbb{R}"
        self.benchmark("Gaussian Integral", latex, iterations=100)


if __name__ == "__main__":
    unittest.main(argv=["first-arg-is-ignored"], verbosity=2)
