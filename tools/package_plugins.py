"""Собрать релизный ZIP ровно из 100 независимых плагинов Adder."""
from pathlib import Path
import hashlib
import json
import zipfile

ROOT = Path(__file__).resolve().parents[1]

def main():
    version = json.loads((ROOT / "app/tauri.conf.json").read_text(encoding="utf-8"))["version"]
    catalog = json.loads((ROOT / "plugins/catalog.json").read_text(encoding="utf-8"))
    assert len(catalog) == 100, "Expected exactly 100 plugins"
    ids, names, commands = set(), set(), set()
    files = {"README.md": ROOT / "plugins/README.md", "catalog.json": ROOT / "plugins/catalog.json"}
    for item in catalog:
        folder = ROOT / "plugins/community" / item["id"]
        manifest = json.loads((folder / "plugin.json").read_text(encoding="utf-8"))
        assert manifest["id"] == item["id"] and manifest["name"] == item["name"]
        assert manifest["version"] == item["version"] and manifest["api"] == 1
        assert manifest["name"].strip(), "Plugin name must not be empty"
        assert manifest["id"] not in ids and manifest["name"] not in names, "Duplicate plugin identity"
        assert not commands.intersection(manifest["commands"]), "Conflicting commands"
        ids.add(manifest["id"]); names.add(manifest["name"]); commands.update(manifest["commands"])
        required = {"plugin.json", "main.adr", "content.json", "START_HERE.md"}
        actual = {p.name for p in folder.iterdir() if p.is_file()}
        assert actual == required, f"Unexpected package contents: {folder}"
        for name in sorted(required):
            files[f"plugins/{manifest['id']}/{name}"] = folder / name
    digest = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in sorted(files.items())}
    output = ROOT / "target/release" / f"Blackwire-Plugins-{version}.zip"
    output.parent.mkdir(parents=True, exist_ok=True)
    prefix = f"Blackwire-Plugins-{version}/"
    metadata = {"version": version, "api": 1, "plugins": len(ids), "sha256": digest}
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, path in sorted(files.items()):
            archive.write(path, prefix + name)
        archive.writestr(prefix + "manifest.json", json.dumps(metadata, ensure_ascii=False, indent=2) + "\n")
    with zipfile.ZipFile(output) as archive:
        assert archive.testzip() is None
        assert len([n for n in archive.namelist() if n.endswith("/plugin.json")]) == 100
        assert len([n for n in archive.namelist() if n.endswith("/START_HERE.md")]) == 100
        for name, expected in digest.items():
            assert hashlib.sha256(archive.read(prefix + name)).hexdigest() == expected
    print(f"{output}: 100 plugins, 100 guides, {len(commands)} unique commands, ZIP checksums OK")

if __name__ == "__main__":
    main()
