#!/usr/bin/env python3
"""Identify only the disposable vkms card through the kernel DRM version ioctl."""
import ctypes
import fcntl
import json
import os
from pathlib import Path
import re
import sys


class Version(ctypes.Structure):
    _fields_ = [('major', ctypes.c_int), ('minor', ctypes.c_int), ('patch', ctypes.c_int),
                ('name_len', ctypes.c_size_t), ('name', ctypes.c_void_p),
                ('date_len', ctypes.c_size_t), ('date', ctypes.c_void_p),
                ('desc_len', ctypes.c_size_t), ('desc', ctypes.c_void_p)]


def main():
    assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Disposable CI only'
    records = []
    selected = []
    for device in sorted(Path('/dev/dri').glob('card*')):
        if not re.fullmatch(r'card[0-9]+', device.name):
            continue
        name = ctypes.create_string_buffer(33)
        version = Version(name_len=32, name=ctypes.addressof(name))
        request = (3 << 30) | (ctypes.sizeof(Version) << 16) | (ord('d') << 8)
        descriptor = os.open(device, os.O_RDONLY | os.O_CLOEXEC)
        try:
            result = bytearray(bytes(version))
            fcntl.ioctl(descriptor, request, result, True)
            size = Version.from_buffer_copy(result).name_len
            assert 0 < size <= 32
            driver = name.raw[:size].decode('ascii')
            records.append(dict(device=str(device), driver=driver))
            if driver == 'vkms':
                selected.append(str(device))
        finally:
            os.close(descriptor)
    Path(sys.argv[1]).write_text(json.dumps(records, indent=2) + '\n')
    assert len(selected) == 1, 'Exactly one real vkms device is required'
    print(selected[0])


if __name__ == '__main__':
    main()
