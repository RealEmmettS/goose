#!/usr/bin/env python3
"""Fail closed unless a binary has the expected ELF or PE machine identity."""

from __future__ import annotations

import argparse
import struct
from pathlib import Path


ELF_MACHINES = {62: "x86_64", 183: "aarch64"}
PE_MACHINES = {0x8664: "x86_64", 0xAA64: "aarch64"}


def read_elf_machine(data: bytes) -> int:
    if len(data) < 20 or data[:4] != b"\x7fELF":
        raise ValueError("not an ELF binary")
    byte_order = {1: "<", 2: ">"}.get(data[5])
    if byte_order is None:
        raise ValueError(f"unsupported ELF byte order {data[5]}")
    return struct.unpack_from(f"{byte_order}H", data, 18)[0]


def read_pe_machine(data: bytes) -> int:
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise ValueError("not a PE binary (missing DOS header)")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if pe_offset + 6 > len(data) or data[pe_offset : pe_offset + 4] != b"PE\0\0":
        raise ValueError("not a PE binary (missing PE signature)")
    return struct.unpack_from("<H", data, pe_offset + 4)[0]


def parse_machine(value: str) -> int:
    try:
        return int(value, 0)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"invalid machine value {value!r}") from error


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--format", choices=("elf", "pe"), required=True)
    parser.add_argument("--machine", type=parse_machine, required=True)
    parser.add_argument("binary", type=Path)
    args = parser.parse_args()

    if not args.binary.is_file() or args.binary.is_symlink():
        parser.error(f"binary is not a regular non-symlink file: {args.binary}")
    data = args.binary.read_bytes()
    try:
        actual = read_elf_machine(data) if args.format == "elf" else read_pe_machine(data)
    except ValueError as error:
        parser.error(str(error))
    if actual != args.machine:
        expected_label = (ELF_MACHINES if args.format == "elf" else PE_MACHINES).get(
            args.machine, "unknown"
        )
        actual_label = (ELF_MACHINES if args.format == "elf" else PE_MACHINES).get(
            actual, "unknown"
        )
        parser.error(
            f"{args.format.upper()} machine mismatch: expected {args.machine:#x} "
            f"({expected_label}), got {actual:#x} ({actual_label})"
        )
    label = (ELF_MACHINES if args.format == "elf" else PE_MACHINES).get(actual, "unknown")
    print(f"{args.binary}: {args.format.upper()} machine {actual:#x} ({label})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
