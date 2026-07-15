#!/usr/bin/env python3
"""Paint a controllable opaque background client inside the disposable X11 smoke display."""

from __future__ import annotations

import argparse
import ctypes
import ctypes.util
import os
import re
import signal
import time
from pathlib import Path


COLOR = re.compile(r"^#[0-9a-fA-F]{6}$")
CW_OVERRIDE_REDIRECT = 1 << 9


class XColor(ctypes.Structure):
    _fields_ = [
        ("pixel", ctypes.c_ulong),
        ("red", ctypes.c_ushort),
        ("green", ctypes.c_ushort),
        ("blue", ctypes.c_ushort),
        ("flags", ctypes.c_char),
        ("pad", ctypes.c_char),
    ]


class XSetWindowAttributes(ctypes.Structure):
    _fields_ = [
        ("background_pixmap", ctypes.c_ulong),
        ("background_pixel", ctypes.c_ulong),
        ("border_pixmap", ctypes.c_ulong),
        ("border_pixel", ctypes.c_ulong),
        ("bit_gravity", ctypes.c_int),
        ("win_gravity", ctypes.c_int),
        ("backing_store", ctypes.c_int),
        ("backing_planes", ctypes.c_ulong),
        ("backing_pixel", ctypes.c_ulong),
        ("save_under", ctypes.c_int),
        ("event_mask", ctypes.c_long),
        ("do_not_propagate_mask", ctypes.c_long),
        ("override_redirect", ctypes.c_int),
        ("colormap", ctypes.c_ulong),
        ("cursor", ctypes.c_ulong),
    ]


def load_x11() -> ctypes.CDLL:
    library = ctypes.util.find_library("X11") or "libX11.so.6"
    x11 = ctypes.CDLL(library)
    display = ctypes.c_void_p
    window = ctypes.c_ulong
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XOpenDisplay.restype = display
    x11.XDefaultScreen.argtypes = [display]
    x11.XDefaultScreen.restype = ctypes.c_int
    x11.XRootWindow.argtypes = [display, ctypes.c_int]
    x11.XRootWindow.restype = window
    x11.XDefaultColormap.argtypes = [display, ctypes.c_int]
    x11.XDefaultColormap.restype = ctypes.c_ulong
    x11.XDisplayWidth.argtypes = [display, ctypes.c_int]
    x11.XDisplayWidth.restype = ctypes.c_int
    x11.XDisplayHeight.argtypes = [display, ctypes.c_int]
    x11.XDisplayHeight.restype = ctypes.c_int
    x11.XParseColor.argtypes = [display, ctypes.c_ulong, ctypes.c_char_p, ctypes.POINTER(XColor)]
    x11.XParseColor.restype = ctypes.c_int
    x11.XAllocColor.argtypes = [display, ctypes.c_ulong, ctypes.POINTER(XColor)]
    x11.XAllocColor.restype = ctypes.c_int
    x11.XCreateSimpleWindow.argtypes = [
        display,
        window,
        ctypes.c_int,
        ctypes.c_int,
        ctypes.c_uint,
        ctypes.c_uint,
        ctypes.c_uint,
        ctypes.c_ulong,
        ctypes.c_ulong,
    ]
    x11.XCreateSimpleWindow.restype = window
    x11.XChangeWindowAttributes.argtypes = [
        display,
        window,
        ctypes.c_ulong,
        ctypes.POINTER(XSetWindowAttributes),
    ]
    x11.XChangeWindowAttributes.restype = ctypes.c_int
    x11.XStoreName.argtypes = [display, window, ctypes.c_char_p]
    x11.XStoreName.restype = ctypes.c_int
    x11.XMapWindow.argtypes = [display, window]
    x11.XMapWindow.restype = ctypes.c_int
    x11.XLowerWindow.argtypes = [display, window]
    x11.XLowerWindow.restype = ctypes.c_int
    x11.XSetWindowBackground.argtypes = [display, window, ctypes.c_ulong]
    x11.XSetWindowBackground.restype = ctypes.c_int
    x11.XClearWindow.argtypes = [display, window]
    x11.XClearWindow.restype = ctypes.c_int
    x11.XSync.argtypes = [display, ctypes.c_int]
    x11.XSync.restype = ctypes.c_int
    x11.XDestroyWindow.argtypes = [display, window]
    x11.XDestroyWindow.restype = ctypes.c_int
    x11.XCloseDisplay.argtypes = [display]
    x11.XCloseDisplay.restype = ctypes.c_int
    return x11


def read_color(path: Path) -> str:
    value = path.read_text(encoding="ascii").strip()
    if not COLOR.fullmatch(value):
        raise ValueError(f"invalid background color command: {value!r}")
    return value.lower()


def write_ready(path: Path, color: str) -> None:
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        os.write(descriptor, f"{color}\n".encode("ascii"))
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    os.replace(temporary, path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--color", required=True)
    parser.add_argument("--command", required=True, type=Path)
    parser.add_argument("--ready", required=True, type=Path)
    arguments = parser.parse_args()
    if not COLOR.fullmatch(arguments.color):
        parser.error("--color must be #RRGGBB")

    x11 = load_x11()
    display = x11.XOpenDisplay(None)
    if not display:
        parser.error("could not open DISPLAY")
    screen = x11.XDefaultScreen(display)
    root = x11.XRootWindow(display, screen)
    colormap = x11.XDefaultColormap(display, screen)

    def pixel_for(color: str) -> int:
        parsed = XColor()
        if not x11.XParseColor(display, colormap, color.encode("ascii"), ctypes.byref(parsed)):
            raise ValueError(f"X11 rejected background color {color}")
        if not x11.XAllocColor(display, colormap, ctypes.byref(parsed)):
            raise ValueError(f"X11 could not allocate background color {color}")
        return int(parsed.pixel)

    running = True
    change_requested = False

    def stop(_signum: int, _frame: object) -> None:
        nonlocal running
        running = False

    def request_change(_signum: int, _frame: object) -> None:
        nonlocal change_requested
        change_requested = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGHUP, stop)
    signal.signal(signal.SIGUSR1, request_change)

    color = arguments.color.lower()
    pixel = pixel_for(color)
    width = x11.XDisplayWidth(display, screen)
    height = x11.XDisplayHeight(display, screen)
    window = x11.XCreateSimpleWindow(display, root, 0, 0, width, height, 0, pixel, pixel)
    if not window:
        parser.error("could not create the X11 smoke background window")
    attributes = XSetWindowAttributes()
    attributes.override_redirect = 1
    x11.XChangeWindowAttributes(
        display, window, CW_OVERRIDE_REDIRECT, ctypes.byref(attributes)
    )
    x11.XStoreName(display, window, b"honk300-smoke-background")
    x11.XMapWindow(display, window)
    x11.XLowerWindow(display, window)
    x11.XSync(display, 0)
    write_ready(arguments.ready, color)
    print(f"X11 smoke background ready: {width}x{height} {color}", flush=True)

    try:
        while running:
            time.sleep(0.05)
            if change_requested:
                change_requested = False
                color = read_color(arguments.command)
                x11.XSetWindowBackground(display, window, pixel_for(color))
                x11.XClearWindow(display, window)
                x11.XLowerWindow(display, window)
                x11.XSync(display, 0)
                write_ready(arguments.ready, color)
                print(f"X11 smoke background changed: {color}", flush=True)
    finally:
        x11.XDestroyWindow(display, window)
        x11.XSync(display, 0)
        x11.XCloseDisplay(display)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
