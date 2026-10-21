import importlib.util
import os
import unittest

spec = importlib.util.spec_from_file_location(
    "build_english", os.path.join(os.path.dirname(os.path.abspath(__file__)), "build_english.py"))
b = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b)


class EnglishTests(unittest.TestCase):
    def test_filters_and_keeps_rank_order(self):
        words = ["the", "The", "to", "x", "a", "it's", "abc1", "www", "fuck", "and", "i"]
        self.assertEqual(b.build(words, 10), ["the", "to", "a", "and", "i"])

    def test_count_limit(self):
        self.assertEqual(b.build(["aa", "bb", "cc"], 2), ["aa", "bb"])


if __name__ == "__main__":
    unittest.main()
