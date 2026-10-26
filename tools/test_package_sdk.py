import argparse
import hashlib
import importlib.util
import platform
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("package_sdk", Path(__file__).with_name("package_sdk.py"))
sdk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sdk)


class SdkCacheTests(unittest.TestCase):
    def test_empty_partial_and_complete_cache_can_be_rebuilt(self):
        for state in ("empty", "partial", "complete"):
            with self.subTest(cache=state), tempfile.TemporaryDirectory(prefix="blackwire-sdk-test-") as temp:
                root = Path(temp).resolve()
                stage = root / "target/sdk"
                stage.mkdir(parents=True)
                legacy_examples = stage / "examples/exploits"
                legacy_examples.mkdir(parents=True)
                readme = root / "README-source.md"
                readme.write_text("current SDK docs", encoding="utf-8")
                binary = root / "checker"
                binary.write_bytes(b"test binary")
                system = {"Windows":"windows", "Linux":"linux", "Darwin":"macos"}[platform.system()]
                if state == "partial":
                    (stage / "README.md").write_text("cached old docs", encoding="utf-8")
                def manifest(files, platforms):
                    return {"version":"0.1.1", "platforms":list(platforms),
                            "sha256":{name:hashlib.sha256(path.read_bytes()).hexdigest() for name,path in files.items()}}
                with patch.object(sdk,"ROOT",root), patch.object(sdk,"STAGE",stage), \
                     patch.object(sdk,"version",lambda:"0.1.1"), \
                     patch.object(sdk,"common_files",lambda:{"README.md":readme}), \
                     patch.object(sdk,"manifest",manifest):
                    args = argparse.Namespace(checker=binary, target=None)
                    if state == "complete":
                        sdk.prepare(args)
                    sdk.prepare(args)
                    sdk.verify(stage)
                    self.assertFalse(legacy_examples.exists(), "stale empty folders must not become invalid plugins")
                    self.assertEqual((stage/"README.md").read_text(encoding="utf-8"), "current SDK docs")
                    self.assertEqual((stage/"bin"/system/sdk.TOOLS[system]).read_bytes(), b"test binary")

    def test_unrelated_file_is_preserved_and_rejected(self):
        with tempfile.TemporaryDirectory(prefix="blackwire-sdk-test-") as temp:
            root = Path(temp).resolve()
            stage = root/"target/sdk"
            stage.mkdir(parents=True)
            unknown=stage/"unrelated.txt"
            unknown.write_text("preserve me", encoding="utf-8")
            binary=root/"checker"
            binary.write_bytes(b"test")
            with patch.object(sdk,"ROOT",root), patch.object(sdk,"STAGE",stage), \
                 patch.object(sdk,"common_files",lambda:{}), \
                 patch.object(sdk,"manifest",lambda files,platforms:{}):
                with self.assertRaisesRegex(ValueError, "Unexpected files"):
                    sdk.prepare(argparse.Namespace(checker=binary,target=None))
            self.assertEqual(unknown.read_text(encoding="utf-8"), "preserve me")


if __name__ == "__main__":
    unittest.main()
