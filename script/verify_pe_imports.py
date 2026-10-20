#!/usr/bin/env python3
"""Require a normal PE32+ DLL import for the pinned Windows settings host.

This checks import metadata, not executable behavior or every PE loader rule.
Bound and delay imports are deliberately unsupported for the pinned SDK output.
The verified companion lease may be
released after GUI readiness only when its DLL was mapped at process startup.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import struct

from verify_binary_architecture import read_pe_machine


def require_load_time_import(data: bytes, required: str) -> None:
    if read_pe_machine(data) not in (0x8664, 0xAA64):
        raise ValueError("settings imports require an x64 or ARM64 PE binary")

    def unpack(fmt: str, offset: int) -> tuple:
        if offset < 0 or offset + struct.calcsize(fmt) > len(data):
            raise ValueError("truncated PE import metadata")
        return struct.unpack_from(fmt, data, offset)

    pe = unpack("<I", 0x3C)[0]
    sections = unpack("<H", pe + 6)[0]
    optional_size = unpack("<H", pe + 20)[0]
    optional = pe + 24
    if optional_size < 112 or optional + optional_size > len(data):
        raise ValueError("truncated PE optional header")
    if unpack("<H", optional)[0] != 0x20B:
        raise ValueError("settings imports require PE32+")
    directory_count = unpack("<I", optional + 108)[0]
    if not 14 <= directory_count <= 16 or optional_size < 112 + directory_count * 8:
        raise ValueError("missing or truncated PE import directories")
    directories = optional + 112
    if unpack("<II", directories + 11 * 8) != (0, 0):
        raise ValueError("bound imports are not supported for Windows settings")
    if unpack("<II", directories + 13 * 8) != (0, 0):
        raise ValueError("delay imports are not supported for Windows settings")

    header_size = unpack("<I", optional + 60)[0]
    section_table = optional + optional_size
    if not 1 <= sections <= 96 or not section_table + sections * 40 <= header_size <= len(data):
        raise ValueError("invalid or truncated PE section table")
    ranges: list[tuple[int, int, int, int]] = []
    for index in range(sections):
        size, rva, raw_size, raw = unpack("<IIII", section_table + index * 40 + 8)
        span = max(size, raw_size)
        if rva < header_size or rva + span > 1 << 32 or raw + raw_size > len(data):
            raise ValueError("PE section is outside the image or file")
        if raw_size and raw < header_size:
            raise ValueError("PE section overlaps its headers")
        for other_rva, other_span, other_raw, other_size in ranges:
            if span and other_span and rva < other_rva + other_span and other_rva < rva + span:
                raise ValueError("overlapping PE virtual sections")
            if raw_size and other_size and raw < other_raw + other_size and other_raw < raw + raw_size:
                raise ValueError("overlapping PE file sections")
        ranges.append((rva, span, raw, raw_size))

    def mapped(rva: int, size: int) -> tuple[int, int]:
        if rva <= 0 or size <= 0 or rva + size > 1 << 32:
            raise ValueError("invalid PE import RVA")
        if rva < header_size and rva + size <= header_size:
            return rva, header_size
        for start, span, raw, raw_size in ranges:
            if start <= rva < start + span and rva + size <= start + raw_size:
                return raw + rva - start, raw + raw_size
        raise ValueError("PE import RVA is not backed by file bytes")

    def import_name(rva: int, prefix: int, kind: str) -> str:
        offset, end = mapped(rva, prefix + 1)
        offset += prefix  # Import-by-name records start with a two-byte Hint.
        terminator = data.find(b"\0", offset, min(end, offset + 512))
        if terminator <= offset:
            raise ValueError(f"empty or unterminated PE import {kind} name")
        try:
            return data[offset:terminator].decode("ascii")
        except UnicodeDecodeError as error:
            raise ValueError(f"non-ASCII PE import {kind} name") from error

    # One image-wide budget includes both tables and their terminators, so many
    # descriptors cannot multiply the per-table bound into excessive work.
    remaining_thunks = 65_536

    def thunks(rva: int) -> tuple[int, ...]:
        nonlocal remaining_thunks
        offset, end = mapped(rva, 8)
        entries: list[int] = []
        for position in range(offset, end - 7, 8):
            if remaining_thunks == 0:
                raise ValueError("oversized PE import thunk tables")
            remaining_thunks -= 1
            value = unpack("<Q", position)[0]
            if value == 0:
                if not entries:
                    raise ValueError("empty PE import thunk table")
                return tuple(entries)
            if value & (1 << 63):
                if value & 0x7FFF_FFFF_FFFF_0000:
                    raise ValueError("invalid PE import ordinal reserved bits")
            else:
                if value > 0x7FFF_FFFF:
                    raise ValueError("invalid PE import Hint/Name RVA")
                import_name(value, 2, "Hint/Name")
            entries.append(value)
        raise ValueError("unterminated PE import thunk table")

    import_rva, import_size = unpack("<II", directories + 8)
    if import_size < 40:
        raise ValueError("missing or truncated normal PE import directory")
    first, _ = mapped(import_rva, import_size)
    names: set[str] = set()
    # The directory size bounds the terminator search; the explicit cap also
    # rejects hostile oversized tables without unbounded descriptor iteration.
    for index in range(min(import_size // 20, 4096)):
        lookup, timestamp, _, name_rva, address_table = unpack("<IIIII", first + index * 20)
        if data[first + index * 20:first + (index + 1) * 20] == bytes(20):
            if required.lower() not in names:
                raise ValueError(f"missing normal/load-time import: {required}")
            return
        if timestamp:
            raise ValueError("bound import descriptors are not supported for Windows settings")
        name = import_name(name_rva, 0, "DLL").lower()
        addresses = thunks(address_table)
        if lookup and lookup != address_table and thunks(lookup) != addresses:
            raise ValueError("mismatched unbound PE import lookup/address tables")
        names.add(name)
    raise ValueError("unterminated or oversized PE import descriptor table")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--require", required=True, dest="required")
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()
    try:
        if not args.binary.is_file() or args.binary.is_symlink():
            raise ValueError("binary is not a regular non-symlink file")
        require_load_time_import(args.binary.read_bytes(), args.required)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(f"{args.binary}: normal/load-time import {args.required}; no bound or delay imports")
