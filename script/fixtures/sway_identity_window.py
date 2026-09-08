#!/usr/bin/env python3
"""Owned XWayland client with no title or advertised process identity, CI only."""
import json
import os
from pathlib import Path
import sys
import time
from Xlib import X, display

assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Disposable CI only'
evidence = Path(sys.argv[1])
connection = display.Display()
screen = connection.screen()
window = screen.root.create_window(80, 80, 280, 160, 0, screen.root_depth,
    X.InputOutput, X.CopyFromParent, background_pixel=screen.white_pixel)
window.set_wm_class('honk300-sway-identity-probe', 'honk300-sway-identity-probe')
# Deliberately omit WM_NAME, _NET_WM_NAME and _NET_WM_PID, as legitimate
# minimal X11 clients do. The real compositor produces the resulting fields.
window.map()
connection.sync()
temporary = evidence / 'identity.new'
temporary.write_text(json.dumps(dict(window=window.id, pid=os.getpid())) + '\n')
temporary.replace(evidence / 'identity.json')
try:
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline and not (evidence / 'identity.stop').exists():
        while connection.pending_events():
            connection.next_event()
        time.sleep(0.01)
finally:
    window.destroy()
    connection.sync()
    connection.close()
    (evidence / 'identity.done').write_text('closed\n')
