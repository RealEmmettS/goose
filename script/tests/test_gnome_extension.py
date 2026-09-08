"""Run deterministic bounds/cancellation checks against the production Shell code."""
from pathlib import Path
import shutil
import subprocess
import unittest


class GnomeExtension(unittest.TestCase):
    @unittest.skipUnless(shutil.which('node'), 'Node is required for Shell JavaScript checks')
    def test_actual_snapshot_bounds_and_cancellation(self):
        root = Path(__file__).resolve().parents[2]
        subprocess.run(['node', '--test', 'script/tests/fixtures/gnome_extension.mjs'],
                       cwd=root, check=True, capture_output=True, text=True, timeout=15)


if __name__ == '__main__':
    unittest.main()
