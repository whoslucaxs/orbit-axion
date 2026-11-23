"""Collect Cargo license declarations and local notice files; no downloads."""
import json, pathlib, subprocess
root = pathlib.Path(__file__).resolve().parents[2]
packages = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--offline", "--filter-platform=x86_64-pc-windows-msvc", "--format-version=1"], cwd=root))["packages"]
output = root / "dist" / "THIRD_PARTY_NOTICES.txt"
output.parent.mkdir(exist_ok=True)
sections = ["Orbit Axion dependency inventory. Cargo license declarations are not a legal audit."]
for package in sorted(packages, key=lambda p: (p["name"], p["version"])):
    if package["source"] is None:
        continue
    directory = pathlib.Path(package["manifest_path"]).parent
    section = [f"{package['name']} {package['version']} | {package.get('license') or 'UNDECLARED'}", package.get("repository") or ""]
    paths = set(directory.glob("LICENSE*")) | set(directory.glob("COPYING*")) | set(directory.glob("NOTICE*"))
    if package.get("license_file"):
        paths.add(directory / package["license_file"])
    for path in sorted(paths):
        if path.is_file():
            section.extend([path.name, path.read_text(encoding="utf-8", errors="replace")])
    sections.append("\n".join(section))
for notice in sorted((root / "assets" / "icons").glob("*-LICENSE.txt")):
    sections.append(notice.name + "\n" + notice.read_text(encoding="utf-8"))
output.write_text("\n\n".join(sections), encoding="utf-8")
print(output)
