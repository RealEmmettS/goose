"""Small PE metadata fixtures; never executable or native GUI evidence."""
import struct


OPTIONAL = 0x98
DIRECTORIES = OPTIONAL + 112
SECTION = OPTIONAL + 240
IMPORTS = 0x200
DLL_NAME = 0x250
LOOKUP = 0x280
ADDRESS_TABLE = 0x2A0
HINT_NAME = 0x2C0


def settings_pe(machine: int = 0x8664, dll_name: bytes = b"honk_settings_accessibility.dll") -> bytearray:
    image = bytearray(0x600)
    image[:2] = b"MZ"
    struct.pack_into("<I", image, 0x3C, 0x80)
    image[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", image, 0x84, machine, 1)
    struct.pack_into("<H", image, 0x94, 240)
    struct.pack_into("<H", image, OPTIONAL, 0x20B)
    struct.pack_into("<I", image, OPTIONAL + 60, 0x200)
    struct.pack_into("<H", image, OPTIONAL + 68, 2)
    struct.pack_into("<I", image, OPTIONAL + 108, 16)
    struct.pack_into("<II", image, DIRECTORIES + 8, 0x1000, 40)
    image[SECTION:SECTION + 8] = b".rdata\0\0"
    struct.pack_into("<IIII", image, SECTION + 8, 0x400, 0x1000, 0x400, 0x200)
    struct.pack_into("<IIIII", image, IMPORTS, 0x1080, 0, 0, 0x1050, 0x10A0)
    image[DLL_NAME:DLL_NAME + len(dll_name) + 1] = dll_name + b"\0"
    struct.pack_into("<Q", image, LOOKUP, 0x10C0)
    struct.pack_into("<Q", image, ADDRESS_TABLE, 0x10C0)
    symbol = b"\0\0honk_a11y_create\0"
    image[HINT_NAME:HINT_NAME + len(symbol)] = symbol
    return image
