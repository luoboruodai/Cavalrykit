#!/usr/bin/env python3
"""Package a verified Windows EXE; validate a candidate before replacing a ZIP."""
import argparse
import hashlib
import json
import re
import tempfile
from pathlib import Path
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED
from verify_windows import verify

ROOT = Path(__file__).resolve().parent.parent
GUIDES = ('INSTALL.zh-CN.txt', 'INSTALL.en.txt', 'RELEASE_NOTES.md')


def release_members(root, executable, version):
    members = {'CavalryKit.exe': executable.read_bytes()}
    for name in ('Install-CavalryKit.cmd', 'Install-CavalryKit.ps1',
                 'WINDOWS-使用说明.txt', 'THIRD-PARTY-NOTICES.txt', *GUIDES):
        path = root / name if name == 'RELEASE_NOTES.md' else root / 'packaging' / name
        data = path.read_bytes()
        if name.endswith(('.cmd', '.ps1')):
            data = data.replace(b'\r\n', b'\n').replace(b'\n', b'\r\n')
        members[name] = data
    for name in GUIDES:
        text = members[name].decode('utf-8')
        if f'CavalryKit {version}' not in text.splitlines()[0]:
            raise ValueError(f'{name}: release version mismatch')
        for link in re.findall(r'\[[^\]]*\]\(([^)]+)\)', text):
            if not link.startswith(('https://', 'http://', '#')) and link not in members:
                raise ValueError(f'{name}: missing offline link target')
        if f'/releases/tag/v{version}' not in text:
            raise ValueError(f'{name}: missing versioned release URL')
    return members


def validate_archive(path, expected):
    with ZipFile(path) as archive:
        names = archive.namelist()
        if len(names) != len(expected) or set(names) != set(expected):
            raise ValueError('ZIP member list differs from the release allowlist')
        if archive.testzip() is not None:
            raise ValueError('ZIP CRC validation failed')
        for name, data in expected.items():
            if archive.read(name) != data:
                raise ValueError(f'ZIP member differs from the frozen input: {name}')


def package(executable, root=ROOT):
    version = re.search(r'^version = "([^"]+)"', (root / 'Cargo.toml').read_text(encoding='utf-8'), re.M).group(1)
    verify(executable, root / 'assets/icon.ico')
    members = release_members(root, executable, version)
    dest = root / 'dist' / f'CavalryKit-{version}-win-x64.zip'
    dest.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix='.package-win-', suffix='.zip', dir=dest.parent, delete=False) as file:
        candidate = Path(file.name)
    try:
        with ZipFile(candidate, 'w', ZIP_DEFLATED, compresslevel=9) as archive:
            for name, data in members.items():
                info = ZipInfo(name, (2026, 10, 14, 0, 0, 0))
                info.compress_type = ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                archive.writestr(info, data)
        validate_archive(candidate, members)
        candidate.replace(dest)
    finally:
        candidate.unlink(missing_ok=True)
    return {'file': dest.name, 'members': list(members),
            'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('executable', type=Path)
    args = parser.parse_args()
    print(json.dumps(package(args.executable), indent=2))


if __name__ == '__main__':
    main()
