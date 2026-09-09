from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('build_leaf_art', ROOT / 'script/build_leaf_art.py')
assert SPEC and SPEC.loader
LEAVES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LEAVES)


class LeafArtworkTests(unittest.TestCase):
    def test_reviewed_svg_assets_and_runtime_geometry_are_synchronized(self) -> None:
        for path, expected in LEAVES.outputs().items():
            with self.subTest(path=path.relative_to(ROOT)):
                self.assertEqual(path.read_text(encoding='utf-8'), expected)

    def test_each_botanical_shape_uses_the_complete_existing_palette(self) -> None:
        palette = json.loads((LEAVES.ART / 'palette.json').read_text(encoding='utf-8'))
        self.assertEqual(palette, {
            'gold': [226, 184, 53], 'orange': [217, 106, 33],
            'red': [169, 59, 42], 'brown': [122, 74, 36],
        })
        all_geometry = set()
        for name in LEAVES.NAMES:
            master = ET.parse(LEAVES.ART / 'masters' / f'{name}.svg').getroot()
            geometry = tuple(p.attrib['d'] for p in list(master)[1:])
            all_geometry.add(geometry)
            for color, rgb in palette.items():
                variant = ET.parse(LEAVES.ART / f'{name}-{color}.svg').getroot()
                self.assertEqual(tuple(p.attrib['d'] for p in list(variant)[1:]), geometry)
                self.assertEqual(list(variant)[1].attrib['fill'], '#%02x%02x%02x' % tuple(rgb))
        self.assertEqual(len(all_geometry), 4)


if __name__ == '__main__':
    unittest.main()
