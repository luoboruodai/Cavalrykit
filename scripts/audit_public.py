#!/usr/bin/env python3
"""Small deterministic preflight: report file/rule only, never matched secret values.
Not a replacement for manual review. No network access and no uploads.
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT_FILES = {'.github/workflows/desktop.yml', '.cargo/config.toml', '.gitignore', '.gitattributes', 'Cargo.toml', 'Cargo.lock', 'build.rs',
              'README.md', 'PRODUCT.md', 'CHANGELOG.md', 'RELEASE_NOTES.md'}
DIRECTORIES = {'src', 'tests', 'assets', 'packaging', 'examples', 'scripts'}
RULES = {
    'private-key': re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    'github-token': re.compile(rb'(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})'),
    'api-token': re.compile(rb'\bsk-(?:proj-)?[A-Za-z0-9_-]{24,}'),
    'credential-url': re.compile(rb'https?://[^\s/@:]+:[^\s/@]+@'),
    'personal-home': re.compile(rb'(?:/Users/|/home/)(?!test(?:/|\\|\b)|build(?:/|\\|\b))[A-Za-z0-9_.-]+/'),
    'password-literal': re.compile(rb'(?i)(?:api_key|password|secret_key)\s*[=:]\s*[\x22\x27][A-Za-z0-9_/-]{16,}[\x22\x27]'),
}


def allowed(path):
    p = Path(path)
    return (str(p) in ROOT_FILES or (len(p.parts) > 1 and p.parts[0] in DIRECTORIES)) and not any(
        part in {'__pycache__', '.DS_Store', '.env', 'private', 'backups', 'PROJECT.md'} or part.endswith('.log')
        for part in p.parts)


def violations(data):
    # Only the build scripts' deterministic, non-personal remap prefix is exempt.
    normalized = data.replace(b'/build/home/', b'/build/toolchain/')
    return [name for name, pattern in RULES.items() if pattern.search(normalized)]


def self_test():
    fixtures = [b'ghp_' + b'a' * 36, b'sk-proj-' + b'x' * 40,
                b'https://' + b'user:password@example.invalid', b'/Users/' + b'private-owner/project/',
                b'-----BEGIN ' + b'PRIVATE KEY-----', b'api_key="' + b'k' * 24 + b'"']
    assert all(violations(data) for data in fixtures), 'scanner accepted a synthetic secret'
    assert not violations(b'/Users/test/Library/Example /build/home/.cargo/registry/src/'), 'false positive on fixture'
    assert not allowed('PROJECT.md') and not allowed('private/notes.txt')
    print('PASS synthetic rejection self-tests')


def git(*args):
    return subprocess.check_output(['git', *args])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--staged', action='store_true')
    parser.add_argument('--history', action='store_true')
    args = parser.parse_args()
    self_test()
    failures = []
    if args.history:
        commits = git('rev-list', '--all').decode().splitlines()
        blobs = {}
        for commit in commits:
            for row in git('ls-tree', '-r', '-z', commit).split(b'\0'):
                if not row:
                    continue
                meta, path = row.split(b'\t', 1)
                blob = meta.split()[2].decode()
                blobs.setdefault(blob, path.decode())
        items = [(name, git('cat-file', 'blob', blob)) for blob, name in blobs.items()]
    elif args.staged:
        names = git('ls-files', '-z').decode().strip('\0').split('\0')
        items = [(name, git('show', ':' + name)) for name in names if name]
    else:
        names = sorted([name for name in ROOT_FILES if Path(name).is_file()] + [
            str(p) for directory in DIRECTORIES for p in Path(directory).rglob('*')
            if p.is_file() and allowed(str(p))])
        items = [(name, Path(name).read_bytes()) for name in names]
    for name, data in items:
        if not args.history and not allowed(name):
            failures.append((name, 'not-in-public-allowlist'))
        failures.extend((name, rule) for rule in violations(data))
    for name, rule in failures:
        print(f'FAIL {name}: {rule}')
    print(f'Checked {len(items)} files/blobs; {len(failures)} findings')
    return bool(failures)


if __name__ == '__main__':
    sys.exit(main())
