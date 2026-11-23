import unittest
from build_msi import rtf

class LicenseTextTests(unittest.TestCase):
    def test_license_text_is_escaped_and_remains_ascii_rtf(self):
        document = rtf("Licença {GPL}\\texto\n©2026")
        self.assertIn(r"Licen\u231?a \{GPL\}\\texto\par", document)
        self.assertIn(r"\u169?2026", document)
        document.encode("ascii")
        self.assertTrue(document.startswith(r"{\rtf1"))
        self.assertTrue(document.endswith("}"))

if __name__ == "__main__":
    unittest.main()
