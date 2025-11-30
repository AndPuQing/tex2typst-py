import unittest
import tex2typst


class TestTexError(unittest.TestCase):
    def test_error(self):
        try:
            tex2typst.tex2typst(
                r"\begin{align*}\int_{{\cal M}_{{\rm ins}}} \left\vert \frac{1}{{\rm det}T_{0}} \right\vert,\end{align*}"
            )
        except Exception as e:
            self.assertIn("Conversion failed:", str(e))
