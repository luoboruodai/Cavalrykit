"""Release ZIP regressions use synthetic EXEs and an isolated copy of public guides."""
import importlib.util
import io
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
from zipfile import ZipFile

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('package_win', HERE / 'package-win.py')
package_win = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package_win)


class ReleasePackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copytree(HERE, self.root / 'packaging', ignore=shutil.ignore_patterns('__pycache__'))
        for name in ('Cargo.toml', 'RELEASE_NOTES.md'):
            shutil.copy2(HERE.parent / name, self.root / name)
        self.exe = self.root / 'fixture.exe'
        self.exe.write_bytes(b'synthetic PE fixture; not executable')
        self.dest = self.root / 'dist/CavalryKit-0.1.1-win-x64.zip'
        self.dest.parent.mkdir()
        self.dest.write_bytes(b'previous valid package fixture')
        self.old = self.dest.read_bytes()
        self.verify = patch.object(package_win, 'verify', return_value={'pass': True})
        self.verify.start()
        self.addCleanup(self.verify.stop)

    def test_complete_package_and_repeatable_bytes(self):
        result = package_win.package(self.exe, self.root)
        first = self.dest.read_bytes()
        self.assertEqual(len(result['members']), 8)
        package_win.package(self.exe, self.root)
        self.assertEqual(first, self.dest.read_bytes())
        with ZipFile(self.dest) as archive:
            self.assertEqual(archive.read('CavalryKit.exe'), self.exe.read_bytes())
            for guide in package_win.GUIDES:
                self.assertIn('CavalryKit 0.1.1', archive.read(guide).decode('utf-8'))

    def test_non_utf8_default_locale_reads_utf8_sources(self):
        real_open = io.open

        def legacy_open(file, mode='r', buffering=-1, encoding=None, *args, **kwargs):
            if 'b' not in mode and encoding is None:
                encoding = 'cp1252'
            return real_open(file, mode, buffering, encoding, *args, **kwargs)

        with patch('io.open', side_effect=legacy_open):
            result = package_win.package(self.exe, self.root)
        self.assertEqual(len(result['members']), 8)

    def test_stale_guide_preserves_previous_package(self):
        guide = self.root / 'packaging/INSTALL.en.txt'
        guide.write_text(guide.read_text(encoding='utf-8').replace('0.1.1', '0.1.0'), encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'version mismatch'):
            package_win.package(self.exe, self.root)
        self.assertEqual(self.dest.read_bytes(), self.old)

    def test_missing_guide_preserves_previous_package(self):
        (self.root / 'packaging/INSTALL.zh-CN.txt').unlink()
        with self.assertRaises(FileNotFoundError):
            package_win.package(self.exe, self.root)
        self.assertEqual(self.dest.read_bytes(), self.old)

    def test_broken_offline_link_preserves_previous_package(self):
        guide = self.root / 'RELEASE_NOTES.md'
        guide.write_text(guide.read_text(encoding='utf-8') + '\n[missing](private-notes.md)\n', encoding='utf-8')
        with self.assertRaisesRegex(ValueError, 'offline link'):
            package_win.package(self.exe, self.root)
        self.assertEqual(self.dest.read_bytes(), self.old)

    def test_candidate_validation_failure_preserves_previous_package(self):
        with patch.object(package_win, 'validate_archive', side_effect=ValueError('CRC fixture failure')):
            with self.assertRaisesRegex(ValueError, 'CRC fixture'):
                package_win.package(self.exe, self.root)
        self.assertEqual(self.dest.read_bytes(), self.old)
        self.assertFalse(list(self.dest.parent.glob('.package-win-*')))

    def test_extra_member_is_rejected(self):
        package_win.package(self.exe, self.root)
        expected = package_win.release_members(self.root, self.exe, '0.1.1')
        with ZipFile(self.dest, 'a') as archive:
            archive.writestr('unexpected-config.json', '{}')
        with self.assertRaisesRegex(ValueError, 'member list'):
            package_win.validate_archive(self.dest, expected)

    def test_changed_binary_is_rejected(self):
        package_win.package(self.exe, self.root)
        self.exe.write_bytes(b'changed synthetic executable')
        expected = package_win.release_members(self.root, self.exe, '0.1.1')
        with self.assertRaisesRegex(ValueError, 'frozen input'):
            package_win.validate_archive(self.dest, expected)


if __name__ == '__main__':
    unittest.main()
