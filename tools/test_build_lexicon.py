import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import build_lexicon as b  # noqa: E402


class ParserTests(unittest.TestCase):
    def test_occ(self):
        occ = b.parse_occ("你 10\n你 12\nbad\nx abc\n好 3\n")
        self.assertEqual(occ, {"你": 12, "好": 3})

    def test_base(self):
        st = b.new_stats()
        text = "ㄝ ㄝ e , big5\n誒 ㄝˋ e4 ,4 big5\n你 ㄋㄧˇ n i3\n壞 abc x y z\n單\n\n"
        self.assertEqual(b.parse_base(text, st), [("誒", "ㄝˋ"), ("你", "ㄋㄧˇ")])
        self.assertEqual(st["base_symbol_skipped"], 1)
        self.assertEqual(st["base_bad_reading"], 1)
        self.assertEqual(st["base_malformed"], 1)

    def test_mappings(self):
        st = b.new_stats()
        text = ("一丁不識 ㄧ ㄉㄧㄥ ㄅㄨˊ ㄕˋ\n喇嘛 ㄌㄚˇ ㄇㄚ˙\n你好 ㄋㄧˇ\n"
                "你好 ab ㄏㄠˇ\n孤\n")
        res = b.parse_mappings(text, st)
        self.assertEqual(len(res), 2)
        self.assertEqual(st["map_length_mismatch"], 1)
        self.assertEqual(st["map_bad_reading"], 1)
        self.assertEqual(st["map_malformed"], 1)

    def test_reading_regex(self):
        for ok in ("ㄋㄧˇ", "ㄧ", "˙ㄇㄚ", "ㄇㄚ˙"):
            self.assertTrue(b.READING_RE.match(ok), ok)
        for bad in ("", "ˇ", "ㄋㄧˇˇ", "ni3", "ㄋ ㄧ"):
            self.assertFalse(b.READING_RE.match(bad), bad)


class BuildTests(unittest.TestCase):
    def make_src(self, d):
        files = {
            "BPMFBase.txt": "你 ㄋㄧˇ n i3 big5\n好 ㄏㄠˇ h3 big5\n好 ㄏㄠˋ h4 big5\nㄝ ㄝ e , big5\n",
            "BPMFMappings.txt": "你好 ㄋㄧˇ ㄏㄠˇ\n",
            "phrase.occ": "你 5\n好 7\n你好 9\n",
            "BPMFPunctuations.txt": "x 1\n",
        }
        for k, v in files.items():
            with open(os.path.join(d, k), "w", encoding="utf-8") as f:
                f.write(v)

    def test_deterministic_and_content(self):
        with tempfile.TemporaryDirectory() as s, tempfile.TemporaryDirectory() as o1, \
                tempfile.TemporaryDirectory() as o2:
            self.make_src(s)
            shas = {b.MCB_REPO: "a" * 40}
            b.build(s, o1, shas, "2026-01-01")
            b.build(s, o2, shas, "2026-01-01")
            for n in ("lexicon.tsv", "SOURCES.txt"):
                with open(os.path.join(o1, n), "rb") as f1, open(os.path.join(o2, n), "rb") as f2:
                    self.assertEqual(f1.read(), f2.read(), n)
            with open(os.path.join(o1, "lexicon.tsv"), encoding="utf-8") as f:
                lines = f.read().splitlines()
            self.assertEqual(lines, sorted(lines))
            self.assertIn("ㄋㄧˇ ㄏㄠˇ\t你好\t9", lines)
            self.assertIn("ㄏㄠˋ\t好\t7", lines)


if __name__ == "__main__":
    unittest.main()
