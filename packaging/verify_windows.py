#!/usr/bin/env python3
"""Verify the shipped PE icon/version/manifest without executing Windows code."""
import argparse
import hashlib
import json
import struct
from pathlib import Path


def verify(executable, icon):
    data = Path(executable).read_bytes()
    ico = Path(icon).read_bytes()
    u16 = lambda offset: struct.unpack_from('<H', data, offset)[0]
    u32 = lambda offset: struct.unpack_from('<I', data, offset)[0]
    assert data[:2] == b'MZ', 'not a PE executable'
    pe = u32(0x3C)
    assert data[pe:pe + 4] == b'PE\0\0'
    assert u16(pe + 4) == 0x8664, 'expected Windows x64'
    optional = pe + 24
    assert u16(optional) == 0x20B, 'expected PE32+'
    assert u16(optional + 68) == 2, 'expected GUI subsystem'
    sections = optional + u16(pe + 20)

    def file_offset(rva):
        for n in range(u16(pe + 6)):
            start = sections + n * 40
            virtual_size, address, raw_size, raw = struct.unpack_from('<IIII', data, start + 8)
            if address <= rva < address + max(virtual_size, raw_size):
                return raw + rva - address
        raise AssertionError(f'unmapped RVA {rva}')

    imports = []
    descriptor = file_offset(u32(optional + 112 + 8))
    while u32(descriptor + 12):
        start = file_offset(u32(descriptor + 12))
        imports.append(data[start:data.index(b'\0', start)].decode('ascii'))
        descriptor += 20
    assert not any(name.upper().startswith(('VCRUNTIME', 'MSVCP')) for name in imports), 'portable EXE requires an external VC++ runtime'

    resource = file_offset(u32(optional + 112 + 2 * 8))
    leaves = {}

    def walk(relative, trail):
        directory = resource + relative
        for n in range(u16(directory + 12) + u16(directory + 14)):
            name, offset = struct.unpack_from('<II', data, directory + 16 + n * 8)
            key = trail + (name,)
            if offset & 0x80000000:
                walk(offset & 0x7FFFFFFF, key)
            else:
                rva, size = struct.unpack_from('<II', data, resource + offset)
                base = file_offset(rva)
                leaves[key] = data[base:base + size]

    walk(0, ())
    types = {key[0] for key in leaves}
    assert {3, 14, 16, 24} <= types, f'missing PE resources: {types}'
    group = next(value for key, value in leaves.items() if key[0] == 14)
    count = struct.unpack_from('<H', ico, 4)[0]
    assert struct.unpack_from('<H', group, 4)[0] == count
    sizes = []
    for n in range(count):
        entry = 6 + 16 * n
        size, offset = struct.unpack_from('<II', ico, entry + 8)
        group_entry = 6 + 14 * n
        resource_id = struct.unpack_from('<H', group, group_entry + 12)[0]
        embedded = next(v for k, v in leaves.items() if k[:2] == (3, resource_id))
        assert embedded == ico[offset:offset + size], f'icon frame {n} differs'
        sizes.append(ico[entry] or 256)
    version = next(v for k, v in leaves.items() if k[0] == 16)
    assert '0.1.1'.encode('utf-16le') in version
    manifest = next(v for k, v in leaves.items() if k[0] == 24)
    assert b'PerMonitorV2' in manifest and b'asInvoker' in manifest
    assert b'/Users/' not in data and b'\\Users\\user\\' not in data, 'local build path leaked'
    return {'pass': True, 'architecture': 'x64', 'subsystem': 'GUI', 'iconFrames': sizes,
            'version': '0.1.1', 'imports': imports, 'externalVCRuntime': False, 'dpi': 'PerMonitorV2', 'runtimeTested': False,
            'sha256': hashlib.sha256(data).hexdigest()}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('executable')
    parser.add_argument('--icon', default='assets/icon.ico')
    args = parser.parse_args()
    print(json.dumps(verify(args.executable, args.icon), indent=2))
