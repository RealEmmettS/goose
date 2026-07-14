from __future__ import annotations

import importlib.util
import struct
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "script" / "verify_binary_architecture.py"
SPEC = importlib.util.spec_from_file_location("binary_architecture", MODULE_PATH)
assert SPEC and SPEC.loader
ARCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ARCH)


class BinaryArchitectureTests(unittest.TestCase):
    def test_reads_little_and_big_endian_elf_machine(self) -> None:
        little = bytearray(20)
        little[:6] = b"\x7fELF\x02\x01"
        struct.pack_into("<H", little, 18, 62)
        self.assertEqual(ARCH.read_elf_machine(bytes(little)), 62)

        big = bytearray(20)
        big[:6] = b"\x7fELF\x02\x02"
        struct.pack_into(">H", big, 18, 183)
        self.assertEqual(ARCH.read_elf_machine(bytes(big)), 183)

    def test_reads_pe_machine_at_declared_header_offset(self) -> None:
        image = bytearray(0x100)
        image[:2] = b"MZ"
        struct.pack_into("<I", image, 0x3C, 0x80)
        image[0x80:0x84] = b"PE\0\0"
        struct.pack_into("<H", image, 0x84, 0xAA64)
        self.assertEqual(ARCH.read_pe_machine(bytes(image)), 0xAA64)

    def test_rejects_malformed_inputs(self) -> None:
        with self.assertRaisesRegex(ValueError, "not an ELF"):
            ARCH.read_elf_machine(b"nope")
        with self.assertRaisesRegex(ValueError, "not a PE"):
            ARCH.read_pe_machine(b"nope")


if __name__ == "__main__":
    unittest.main()
