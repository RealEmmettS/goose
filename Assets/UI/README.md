# Goose application and control-surface artwork

`settings/assets/icon.png` is the approved full-color Mac and Linux application logo.
Windows uses the existing multi-resolution `honk300-app.ico` application badge.
ADR 0054 makes each platform's tray or menu icon use its own application artwork.

- Windows loads resource 1 from its own executable, the same `honk300-app.ico` used by
  its application and settings window, at the native small-icon size.
- Linux calls `honk_control::icon::tray_pixmap` once at tray creation. The complete image
  and transparent margin are retained. StatusNotifier uses straight-alpha ARGB in network byte order.
- macOS packages the exact source as `Goose-status.png` before signing. AppKit displays its
  silhouette at 18 points with template tinting for the current menu-bar appearance.
- Native control names remain **Goose controls**, and Configure, Update and graceful Quit
  continue through the existing shared services.

The older `honk300-status-goose.svg` and its `@2x.png` raster remain as historical source
artwork, and `honk300-app.svg` records the Windows badge design. The old status mark is
no longer the runtime tray source. It was generated with Quiver AI `arrow-1.1` on
2026-07-17 (generation
`942edcac957b43dfae78cb52402faa30`) and normalized to a two-path monochrome silhouette.

`honk300-app.ico` is the approved multi-resolution Windows application resource referenced
by `honk300-app.rc`. Keep each platform's control surface tied to its actual application
artwork instead of introducing a separately redrawn or recolored logo.
