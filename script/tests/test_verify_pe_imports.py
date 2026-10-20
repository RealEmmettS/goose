from __future__ import annotations

from pathlib import Path
import struct
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "script"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from pe_fixtures import ADDRESS_TABLE, DIRECTORIES, DLL_NAME, HINT_NAME, IMPORTS, LOOKUP, OPTIONAL, SECTION, settings_pe
from verify_pe_imports import require_load_time_import

REQUIRED = "honk_settings_accessibility.dll"


class PeImportsTests(unittest.TestCase):
    def test_accepts_normal_import_for_x64_and_arm64(self):
        for machine in (0x8664, 0xAA64):
            with self.subTest(machine=machine):
                require_load_time_import(settings_pe(machine), REQUIRED)

    def test_dll_names_are_case_insensitive(self):
        require_load_time_import(settings_pe(dll_name=REQUIRED.upper().encode()), REQUIRED)

    def test_rejects_missing_required_dll(self):
        with self.assertRaisesRegex(ValueError, "missing normal/load-time import"):
            require_load_time_import(settings_pe(dll_name=b"other.dll"), REQUIRED)

    def test_rejects_missing_normal_directory(self):
        image = settings_pe()
        struct.pack_into("<II", image, DIRECTORIES + 8, 0, 0)
        with self.assertRaisesRegex(ValueError, "missing or truncated normal"):
            require_load_time_import(image, REQUIRED)

    def test_rejects_delay_only_and_mixed_imports(self):
        for normal in (False, True):
            with self.subTest(normal=normal):
                image = settings_pe()
                if not normal:
                    struct.pack_into("<II", image, DIRECTORIES + 8, 0, 0)
                struct.pack_into("<II", image, DIRECTORIES + 13 * 8, 0x1100, 64)
                with self.assertRaisesRegex(ValueError, "delay imports"):
                    require_load_time_import(image, REQUIRED)

    def test_rejects_inconsistent_delay_directory_fields(self):
        for rva, size in ((0x1100, 0), (0, 64)):
            image = settings_pe()
            struct.pack_into("<II", image, DIRECTORIES + 13 * 8, rva, size)
            with self.assertRaisesRegex(ValueError, "delay imports"):
                require_load_time_import(image, REQUIRED)

    def test_rejects_truncated_headers_sections_and_payload(self):
        for length in (0, 0x3F, 0x85, 0x97, 0xA0, SECTION + 39, 0x5FF):
            with self.subTest(length=length), self.assertRaises(ValueError):
                require_load_time_import(settings_pe()[:length], REQUIRED)

    def test_rejects_unsupported_machine_magic_and_directory_count(self):
        for offset, fmt, value in ((0x84, "<H", 0x14C), (OPTIONAL, "<H", 0x10B),
                                   (OPTIONAL + 108, "<I", 13), (OPTIONAL + 108, "<I", 17),
                                   (0x94, "<H", 112)):
            with self.subTest(offset=offset, value=value):
                image = settings_pe()
                struct.pack_into(fmt, image, offset, value)
                with self.assertRaises(ValueError):
                    require_load_time_import(image, REQUIRED)

    def test_rejects_out_of_file_and_unmapped_rvas(self):
        for offset in (DIRECTORIES + 8, IMPORTS, IMPORTS + 12, IMPORTS + 16):
            with self.subTest(offset=offset):
                image = settings_pe()
                struct.pack_into("<I", image, offset, 0xFFFFFFF0)
                with self.assertRaisesRegex(ValueError, "RVA"):
                    require_load_time_import(image, REQUIRED)

    def test_rejects_virtual_only_name_bytes(self):
        image = settings_pe()
        struct.pack_into("<I", image, SECTION + 8, 0x800)
        struct.pack_into("<I", image, IMPORTS + 12, 0x1500)
        with self.assertRaisesRegex(ValueError, "not backed by file"):
            require_load_time_import(image, REQUIRED)

    def test_rejects_import_directory_beyond_section(self):
        image = settings_pe()
        struct.pack_into("<I", image, DIRECTORIES + 12, 0x401)
        with self.assertRaisesRegex(ValueError, "not backed by file"):
            require_load_time_import(image, REQUIRED)

    def test_requires_descriptor_terminator_even_after_required_dll(self):
        image = settings_pe()
        image[IMPORTS + 20:IMPORTS + 40] = image[IMPORTS:IMPORTS + 20]
        with self.assertRaisesRegex(ValueError, "unterminated.*descriptor"):
            require_load_time_import(image, REQUIRED)

    def test_rejects_descriptor_table_too_short_for_terminator(self):
        image = settings_pe()
        struct.pack_into("<I", image, DIRECTORIES + 12, 20)
        with self.assertRaisesRegex(ValueError, "truncated normal"):
            require_load_time_import(image, REQUIRED)

    def test_validates_descriptors_after_required_dll(self):
        image = settings_pe()
        struct.pack_into("<I", image, DIRECTORIES + 12, 60)
        image[IMPORTS + 20:IMPORTS + 40] = image[IMPORTS:IMPORTS + 20]
        struct.pack_into("<I", image, IMPORTS + 20 + 12, 0xFFFFFFF0)
        with self.assertRaisesRegex(ValueError, "RVA"):
            require_load_time_import(image, REQUIRED)

    def test_rejects_empty_unterminated_or_non_ascii_dll_name(self):
        for kind in ("empty", "unterminated", "non-ascii"):
            with self.subTest(kind=kind):
                image = settings_pe()
                if kind == "unterminated":
                    struct.pack_into("<I", image, IMPORTS + 12, 0x13FC)
                    image[-4:] = b"abcd"
                else:
                    image[DLL_NAME:DLL_NAME + 2] = b"\0\0" if kind == "empty" else b"\xff\0"
                with self.assertRaisesRegex(ValueError, "DLL name"):
                    require_load_time_import(image, REQUIRED)

    def test_rejects_empty_import_thunk_tables(self):
        for offset in (LOOKUP, ADDRESS_TABLE):
            with self.subTest(offset=offset):
                image = settings_pe()
                struct.pack_into("<Q", image, offset, 0)
                with self.assertRaisesRegex(ValueError, "empty PE import thunk"):
                    require_load_time_import(image, REQUIRED)

    def test_accepts_original_thunk_fallback(self):
        for machine in (0x8664, 0xAA64):
            image = settings_pe(machine)
            struct.pack_into("<I", image, IMPORTS, 0)
            require_load_time_import(image, REQUIRED)

    def test_accepts_multiple_named_and_ordinal_imports(self):
        for machine in (0x8664, 0xAA64):
            for ordinal in (0, 42, 0xFFFF):
                with self.subTest(machine=machine, ordinal=ordinal):
                    image = settings_pe(machine)
                    for offset in (LOOKUP, ADDRESS_TABLE):
                        struct.pack_into("<Q", image, offset + 8, (1 << 63) | ordinal)
                    require_load_time_import(image, REQUIRED)

    def test_rejects_unmapped_or_invalid_hint_name_rvas_in_every_thunk(self):
        for machine in (0x8664, 0xAA64):
            for table in (LOOKUP, ADDRESS_TABLE):
                for index in (0, 1):
                    for value in (0x7FFF_FFFF, 0x8000_0000, 0x1_0000_10C0):
                        with self.subTest(machine=machine, table=table, index=index, value=value):
                            image = settings_pe(machine)
                            struct.pack_into("<Q", image, table + index * 8, value)
                            with self.assertRaisesRegex(ValueError, "RVA"):
                                require_load_time_import(image, REQUIRED)

    def test_rejects_unterminated_ilt_iat_and_fallback_at_section_end(self):
        for machine in (0x8664, 0xAA64):
            for field in (IMPORTS, IMPORTS + 16):
                for fallback in (False, True):
                    if fallback and field == IMPORTS:
                        continue
                    with self.subTest(machine=machine, field=field, fallback=fallback):
                        image = settings_pe(machine)
                        if fallback:
                            struct.pack_into("<I", image, IMPORTS, 0)
                        struct.pack_into("<I", image, field, 0x13F8)
                        struct.pack_into("<Q", image, len(image) - 8, 0x10C0)
                        # Bytes outside the section must not supply its terminator.
                        image.extend(bytes(8))
                        with self.assertRaisesRegex(ValueError, "unterminated.*thunk"):
                            require_load_time_import(image, REQUIRED)

    def test_rejects_truncated_empty_unterminated_or_non_ascii_hint_name(self):
        for machine in (0x8664, 0xAA64):
            for kind in ("truncated", "empty", "unterminated", "non-ascii"):
                with self.subTest(machine=machine, kind=kind):
                    image = settings_pe(machine)
                    if kind == "truncated":
                        struct.pack_into("<Q", image, LOOKUP, 0x13FF)
                    elif kind == "unterminated":
                        struct.pack_into("<Q", image, LOOKUP, 0x13FC)
                        image[-4:] = b"\0\0ab"
                    else:
                        image[HINT_NAME + 2:HINT_NAME + 4] = b"\0\0" if kind == "empty" else b"\xff\0"
                    with self.assertRaises(ValueError):
                        require_load_time_import(image, REQUIRED)

    def test_rejects_ordinal_reserved_bits(self):
        for machine in (0x8664, 0xAA64):
            for table in (LOOKUP, ADDRESS_TABLE):
                image = settings_pe(machine)
                struct.pack_into("<Q", image, table, (1 << 63) | (1 << 16) | 42)
                with self.assertRaisesRegex(ValueError, "ordinal reserved bits"):
                    require_load_time_import(image, REQUIRED)

    def test_rejects_mismatched_unbound_ilt_and_iat(self):
        for machine in (0x8664, 0xAA64):
            image = settings_pe(machine)
            struct.pack_into("<Q", image, ADDRESS_TABLE, (1 << 63) | 42)
            with self.assertRaisesRegex(ValueError, "mismatched unbound"):
                require_load_time_import(image, REQUIRED)

    def test_rejects_bound_import_metadata(self):
        for machine in (0x8664, 0xAA64):
            for directory in (False, True):
                with self.subTest(machine=machine, directory=directory):
                    image = settings_pe(machine)
                    if directory:
                        struct.pack_into("<II", image, DIRECTORIES + 11 * 8, 0x1100, 64)
                    else:
                        struct.pack_into("<I", image, IMPORTS + 4, 0x12345678)
                    with self.assertRaisesRegex(ValueError, "bound import"):
                        require_load_time_import(image, REQUIRED)

    def test_rejects_oversized_thunk_tables_with_finite_budget(self):
        image = settings_pe()
        image.extend(struct.pack("<Q", (1 << 63) | 42) * 65_536 + bytes(8))
        raw_size = len(image) - 0x200
        struct.pack_into("<IIII", image, SECTION + 8, raw_size, 0x1000, raw_size, 0x200)
        struct.pack_into("<I", image, IMPORTS, 0)
        struct.pack_into("<I", image, IMPORTS + 16, 0x1400)
        with self.assertRaisesRegex(ValueError, "oversized.*thunk"):
            require_load_time_import(image, REQUIRED)

    def test_rejects_overlapping_sections(self):
        image = settings_pe()
        struct.pack_into("<H", image, 0x86, 2)
        image[SECTION + 40:SECTION + 80] = image[SECTION:SECTION + 40]
        with self.assertRaisesRegex(ValueError, "overlapping PE"):
            require_load_time_import(image, REQUIRED)


if __name__ == "__main__":
    unittest.main()
