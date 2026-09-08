"""Exercise Cargo's real file selection for the embedded desktop companion."""
from pathlib import Path
import shutil
import subprocess
import unittest


class CargoSourcePackage(unittest.TestCase):
    @unittest.skipUnless(shutil.which('cargo'), 'Cargo is required for source-package selection')
    def test_embedded_companions_are_in_the_actual_source_package(self):
        root = Path(__file__).resolve().parents[2]
        result = subprocess.run(
            ['cargo', 'package', '--locked', '--list', '--allow-dirty', '--no-verify', '-p', 'honk300'],
            cwd=root, capture_output=True, text=True, timeout=60, check=True)
        files = {line.replace('\\', '/') for line in result.stdout.splitlines()}
        self.assertIn('integrations/kwin/contents/code/main.js', files)
        self.assertIn('integrations/gnome/extension.js', files)
        self.assertIn('integrations/gnome/metadata.json', files)


if __name__ == '__main__':
    unittest.main()
