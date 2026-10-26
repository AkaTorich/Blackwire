"""Build the SDK installed with Blackwire and the multi-platform release ZIP."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
STAGE = ROOT / "target" / "sdk"
TOOLS = {"windows": "plugin-check.exe", "linux": "plugin-check", "macos": "plugin-check"}
DOCS = ("SDK.md", "ADDER.md", "COMMANDS.md", "GAME.md", "HACK.md", "STORY.md")


def run(args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


def version():
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    value = cargo["workspace"]["package"]["version"]
    config = json.loads((ROOT / "app/tauri.conf.json").read_text(encoding="utf-8"))
    if config["version"] != value:
        raise ValueError("Cargo and Tauri versions differ")
    return value


def common_files():
    files = {"README.md": ROOT / "sdk/README.md", "Blackwire.chm": ROOT / "Blackwire.chm"}
    for name in DOCS:
        files["docs/" + name] = ROOT / "docs" / name
    for folder in ("examples", "schema"):
        for source in sorted((ROOT / "sdk" / folder).rglob("*")):
            if source.is_file():
                files[source.relative_to(ROOT / "sdk").as_posix()] = source
    for source in files.values():
        if not source.is_file():
            raise FileNotFoundError(source)
    return files


def manifest(files, platforms):
    source = (ROOT / "core/src/plugins/mod.rs").read_text(encoding="utf-8")
    api = int(re.search(r"API_VERSION\s*:\s*u32\s*=\s*(\d+)", source)[1])
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    return {"name": "Blackwire SDK", "version": version(), "api": api, "commit": commit,
            "platforms": sorted(platforms),
            "sha256": {name: hashlib.sha256(path.read_bytes()).hexdigest()
                       for name, path in sorted(files.items())}}


def checker(target):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version=1", "--no-deps"], cwd=ROOT))
    target_dir = Path(metadata["target_directory"])
    host = next(line.split(": ", 1)[1] for line in subprocess.check_output(
        ["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
    build = ["cargo", "build", "--release", "--locked", "-p", "blackwire-core", "--bin", "plugin-check"]
    if target == "universal-apple-darwin":
        arches = ("aarch64-apple-darwin", "x86_64-apple-darwin")
        for arch in arches:
            run(build + ["--target", arch])
        result = target_dir / target / "release/plugin-check"
        result.parent.mkdir(parents=True, exist_ok=True)
        run(["lipo", "-create", *(target_dir / arch / "release/plugin-check" for arch in arches),
             "-output", result])
        return "macos", result
    target = target or host
    if target == host:
        run(build)
        folder = target_dir / "release"
    else:
        run(build + ["--target", target])
        folder = target_dir / target / "release"
    system = "windows" if "windows" in target else "macos" if "apple-darwin" in target else "linux"
    return system, folder / TOOLS[system]


def prepare(args):
    if args.checker:
        system = {"Windows": "windows", "Linux": "linux", "Darwin": "macos"}[platform.system()]
        binary = args.checker.resolve()
    else:
        system, binary = checker(args.target or os.environ.get("TAURI_ENV_TARGET_TRIPLE"))
    if not binary.is_file():
        raise FileNotFoundError(binary)
    files = common_files()
    files[f"bin/{system}/{TOOLS[system]}"] = binary
    data = manifest(files, [system])
    # Only the generated staging directory inside this project's target is managed here.
    expected = ROOT.resolve() / "target" / "sdk"
    if STAGE.resolve() != expected or (ROOT / "target").is_symlink():
        raise ValueError("SDK staging path must stay inside the project target directory")
    if STAGE.exists():
        entries = list(STAGE.rglob("*"))
        if any(path.is_symlink() for path in entries):
            raise ValueError("SDK staging directory must not contain symbolic links")
        old_manifest = STAGE / "manifest.json"
        previous = (json.loads(old_manifest.read_text(encoding="utf-8"))["sha256"]
                    if old_manifest.is_file() else {})
        actual = {p.relative_to(STAGE).as_posix() for p in entries if p.is_file()}
        # Rust's CI cache can restore empty directories or only some generated SDK files.
        # Recreate known package files, while preserving the refusal to touch unrelated files.
        if actual - set(previous) - set(files) - {"manifest.json"}:
            raise ValueError("Unexpected files in SDK staging directory")
        for name in previous:
            obsolete = (STAGE / name).resolve()
            if not obsolete.is_relative_to(expected):
                raise ValueError("Invalid path in SDK staging manifest")
            if name not in files and obsolete.is_file():
                obsolete.unlink()
    for name, source in files.items():
        dest = STAGE / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, dest)
    # Empty cached example folders would be mistaken for plugins by the loader.
    for directory in sorted((p for p in STAGE.rglob("*") if p.is_dir()),
                            key=lambda p: len(p.parts), reverse=True):
        if not any(directory.iterdir()):
            directory.rmdir()
    (STAGE / "manifest.json").write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    verify(STAGE)
    print(f"SDK {version()}: {len(files)} files staged for {system}")


def verify(folder):
    data = json.loads((folder / "manifest.json").read_text(encoding="utf-8"))
    for name, expected in data["sha256"].items():
        path = (folder / name).resolve()
        if not path.is_relative_to(folder.resolve()):
            raise ValueError("Invalid SDK path")
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"SDK checksum mismatch: {name}")
    for system in data["platforms"]:
        binary = folder / "bin" / system / TOOLS[system]
        if not binary.is_file():
            raise FileNotFoundError(binary)
    print("SDK file checksums: OK")


def archive(args):
    files = common_files()
    for system, name in TOOLS.items():
        binary = args.tools / system / name
        if not binary.is_file():
            raise FileNotFoundError(binary)
        files[f"bin/{system}/{name}"] = binary
    data = manifest(files, TOOLS)
    output = ROOT / "target/release" / f"Blackwire-SDK-{version()}.zip"
    output.parent.mkdir(parents=True, exist_ok=True)
    prefix = f"Blackwire-SDK-{version()}/"
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name, path in sorted(files.items()):
            z.write(path, prefix + name)
        z.writestr(prefix + "manifest.json", json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    with zipfile.ZipFile(output) as z:
        if z.testzip() is not None:
            raise ValueError("SDK ZIP integrity check failed")
        for name, digest in data["sha256"].items():
            if hashlib.sha256(z.read(prefix + name)).hexdigest() != digest:
                raise ValueError(f"Archived SDK checksum mismatch: {name}")
    print(f"Release SDK ZIP: {output} ({output.stat().st_size} bytes)")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    prep = commands.add_parser("prepare")
    prep.add_argument("--target")
    prep.add_argument("--checker", type=Path, help="Use an already built checker for a packaging check")
    prep.set_defaults(func=prepare)
    check = commands.add_parser("verify")
    check.add_argument("folder", type=Path, nargs="?", default=STAGE)
    check.set_defaults(func=lambda args: verify(args.folder))
    pack = commands.add_parser("archive")
    pack.add_argument("--tools", type=Path, required=True, help="Downloaded SDK tools: windows/, linux/, macos/")
    pack.set_defaults(func=archive)
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
