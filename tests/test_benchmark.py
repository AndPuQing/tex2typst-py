import unittest
import time
import tex2typst


class TestPerformance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        print("\n[Setup] Initializing QuickJS Runtime...")
        start_t = time.perf_counter()
        cls.converter = tex2typst.Tex2Typst()
        init_time = (time.perf_counter() - start_t) * 1000
        print(f"[Setup] Runtime ready. Init time: {init_time:.2f} ms")

    def benchmark(self, name, latex_input, iterations=10000):
        converter = self.converter

        for _ in range(100):
            converter.convert(latex_input)

        start_time = time.perf_counter()

        for _ in range(iterations):
            converter.convert(latex_input)

        end_time = time.perf_counter()

        total_time = end_time - start_time
        avg_latency_ms = (total_time / iterations) * 1000
        throughput_qps = iterations / total_time

        print(f"\n--- Benchmark: {name} ---")
        print(f"Iterations : {iterations}")
        print(f"Total Time : {total_time:.4f} s")
        print(f"Latency    : {avg_latency_ms:.4f} ms/op")
        print(f"Throughput : {throughput_qps:.0f} ops/sec")

        self.assertGreater(throughput_qps, 10, "Throughput is surprisingly low!")

    def test_perf_simple(self):
        latex = "\\alpha"
        self.benchmark("Simple Token", latex, iterations=5000)

    def test_perf_medium(self):
        latex = "\\frac{-b \pm \sqrt{b^2 - 4ac}}{2a}"
        self.benchmark("Quadratic Formula", latex, iterations=2000)

    def test_perf_complex(self):
        latex = r"\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \sqrt{\\pi} \\quad \\text{where } x \\in \\mathbb{R}"
        self.benchmark("Gaussian Integral", latex, iterations=1000)


if __name__ == "__main__":
    unittest.main(argv=["first-arg-is-ignored"], verbosity=2)
