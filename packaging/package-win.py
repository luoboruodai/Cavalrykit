#!/usr/bin/env python3
"""Package a verified cross-built Windows EXE using an explicit, reproducible member map."""
import argparse
import hashlib
import json
import re
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
from verify_windows import verify

ROOT = Path(__file__).resolve().parent.parent

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('executable', type=Path)
    args = parser.parse_args()
    version = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M).group(1)
    verify(args.executable, ROOT / 'assets/icon.ico')
    members = {'CavalryKit.exe': args.executable}
    for name in ['Install-CavalryKit.cmd', 'Install-CavalryKit.ps1', 'WINDOWS-使用说明.txt', 'THIRD-PARTY-NOTICES.txt']:
        members[name] = ROOT / 'packaging' / name
    dest = ROOT / 'dist' / f'CavalryKit-{version}-win-x64.zip'
    dest.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(dest, 'w', ZIP_DEFLATED, compresslevel=9) as archive:
        for name, path in members.items():
            data = path.read_bytes()
            if name.endswith(('.cmd', '.ps1')):
                data = data.replace(b'\r\n', b'\n').replace(b'\n', b'\r\n')
            info = ZipInfo(name, (2026, 10, 12, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    with ZipFile(dest) as archive:
        assert set(archive.namelist()) == set(members)
        assert archive.read('CavalryKit.exe') == args.executable.read_bytes()
    print(json.dumps({'file': dest.name, 'members': list(members), 'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()}, indent=2))

if __name__ == '__main__':
    main()
