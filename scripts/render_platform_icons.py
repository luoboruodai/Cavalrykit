#!/usr/bin/env python3
"""Validate and assemble the frozen, human-approved optical-size icon frames.

Kept under the historical script name for compatibility. This is deliberately
NOT a renderer: a normal build must not redraw an obsolete metallic style over
newly approved artwork. New designs need separate review before replacing the
canonical PNGs and their manifest.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def verify_frames(root=ROOT):
    design = json.loads((root / 'assets/icon-design.json').read_text())
    if design.get('revision') != 'icon-final-inset-v6':
        raise ValueError('Unexpected design revision; new artwork needs explicit review')
    for section in ('canonical_frames_sha256', 'source_frames_sha256'):
        for name, expected in design[section].items():
            path = (root / name).resolve()
            if root.resolve() not in path.parents:
                raise ValueError('Frame path escapes repository')
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            if actual != expected:
                raise ValueError(f'Approved frame mismatch: {name}')
    return design


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='verify frames and assembled assets without writing')
    args = parser.parse_args()
    verify_frames()
    command = ['cargo', 'run', '--locked', '--example', 'import_icon']
    if args.check:
        command.extend(['--', '--check'])
    subprocess.run(command, cwd=ROOT, check=True)
    print('Approved V6 frames verified; no artwork was redrawn.')


if __name__ == '__main__':
    main()
