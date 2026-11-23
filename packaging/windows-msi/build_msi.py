"""Build the standard WiX MSI; requires Python 3.9+ and .NET runtime 6/8+."""
import hashlib
import io
import pathlib
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import xml.etree.ElementTree as ET
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
VERSION = "5.0.2"
PACKAGES = {
    "wix": "f30ef0c74e2a986126539c5780be93ac24e8136eaf723b1937b26272703ae173",
    "wixtoolset.ui.wixext": "5ef2c707614b9f70b6bbadd2d4abcb4124efee215e9b16bfbc80113079a604c7",
}

def toolchain():
    if not shutil.which("dotnet"):
        raise RuntimeError("Install the .NET 8 runtime to build the MSI")
    cache = ROOT / "target" / "wix" / VERSION
    for name, digest in PACKAGES.items():
        directory = cache / name
        marker = directory / ".verified"
        if marker.is_file() and marker.read_text() == digest:
            continue
        url = f"https://api.nuget.org/v3-flatcontainer/{name}/{VERSION}/{name}.{VERSION}.nupkg"
        with urllib.request.urlopen(url, timeout=90) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != digest:
            raise RuntimeError(f"WiX package checksum mismatch: {name}")
        directory.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            for entry in archive.infolist():
                target = (directory / entry.filename).resolve()
                if not target.is_relative_to(directory.resolve()):
                    raise RuntimeError("Unsafe WiX archive path")
                if not entry.is_dir():
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(archive.read(entry))
        marker.write_text(digest)
    return ["dotnet", str(cache / "wix/tools/net6.0/any/wix.dll")], cache / "wixtoolset.ui.wixext/wixext5/WixToolset.UI.wixext.dll"

def rtf(text):
    escaped = []
    for character in text:
        if character in "\\{}":
            escaped.append("\\" + character)
        elif character == "\n":
            escaped.append("\\par\n")
        elif ord(character) > 127:
            code = ord(character)
            escaped.append(f"\\u{code if code < 32768 else code - 65536}?")
        else:
            escaped.append(character)
    return r"{\rtf1\ansi\deff0{\fonttbl{\f0 Segoe UI;}}\f0\fs18 " + "".join(escaped) + "}"

def build(exe, output, version):
    exe, output = pathlib.Path(exe).resolve(), pathlib.Path(output).resolve()
    if not exe.is_file():
        raise RuntimeError("Build the EXE first")
    notices = ROOT / "dist/THIRD_PARTY_NOTICES.txt"
    if not notices.is_file():
        raise RuntimeError("Run collect_notices.py first")
    command, extension = toolchain()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="orbit-wix-", dir=output.parent) as scratch:
        scratch = pathlib.Path(scratch)
        license_rtf = scratch / "license.rtf"
        license_rtf.write_text(rtf((ROOT / "LICENSE").read_text(encoding="utf-8")), encoding="ascii")
        subprocess.run(command + ["build", str(ROOT / "packaging/windows-msi/Package.wxs"), "-arch", "x64", "-ext", str(extension), "-culture", "pt-BR", "-d", f"Version={version}", "-d", f"Root={ROOT}", "-d", f"Exe={exe}", "-d", f"LicenseRtf={license_rtf}", "-o", str(output), "-pdbtype", "none"], check=True)
        subprocess.run(command + ["msi", "validate", str(output)], check=True)
        extracted = scratch / "extracted"
        manifest = scratch / "package.wxs"
        subprocess.run(command + ["msi", "decompile", str(output), "-x", str(extracted), "-o", str(manifest)], check=True)
        for file_id, source in {"OrbitExecutable": exe, "OrbitLicense": ROOT / "LICENSE", "OrbitNotices": notices}.items():
            actual = extracted / "File" / file_id
            if hashlib.sha256(source.read_bytes()).digest() != hashlib.sha256(actual.read_bytes()).digest():
                raise RuntimeError(f"MSI payload mismatch: {file_id}")
        ns = {"w": "http://wixtoolset.org/schemas/v4/wxs"}
        tree = ET.parse(manifest)
        dialogs = {node.get("Id") for node in tree.findall(".//w:Dialog", ns)}
        required = {"WelcomeDlg", "LicenseAgreementDlg", "InstallScopeDlg", "InstallDirDlg", "VerifyReadyDlg", "ProgressDlg", "ExitDialog"}
        if not required.issubset(dialogs) or "OrbitWelcome" in dialogs:
            raise RuntimeError("Standard installer dialogs are missing")
    print(output)

if __name__ == "__main__":
    build(*sys.argv[1:])
