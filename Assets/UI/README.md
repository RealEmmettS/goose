# Honk300 shared control-surface icon

`honk300-status-goose.svg` is the canonical, platform-neutral source for the small goose mark used
by operating-system control surfaces. macOS uses it now for the menu-bar status item. Future
Windows notification-area and Linux StatusNotifier/AppIndicator integrations should derive their
native-size assets from this file instead of redrawing or copying a platform-specific raster.
`honk300-status-goose@2x.png` is the 36×36 transparent runtime representation used by AppKit at
18 points; retaining the PNG avoids depending on raw SVG decoding across the app's macOS 11+
support range.

## Provenance

- Generated with Quiver AI `arrow-1.1` on 2026-07-17 (generation
  `942edcac957b43dfae78cb52402faa30`).
- Prompt: “Tiny side-profile desktop goose silhouette standing upright with a long curved neck,
  rounded body, small wing, pointed beak, and two tiny webbed feet, recognizable at 16 pixels,
  fully transparent background.”
- Direction: a macOS template icon, one solid silhouette, compact square composition, no text,
  gradients, shading, background rectangle, or clipped parts.
- The generated SVG was normalized into a transparent two-path monochrome source and its unused
  background polygon was removed. No runtime network request is made.

## Integration contract

- Treat the black artwork as a mask/template. The operating system supplies the light/dark tint;
  do not bake a menu-bar background or color theme into the asset.
- Preserve the accessible control name **Honk300 controls** independently of the image.
- Generate platform raster sizes deterministically from this SVG when a platform cannot consume
  SVG directly. Keep the SVG tracked as the source of truth and verify 16–20 point/pixel output.
- Regenerate the AppKit representation with
  `magick -background none -density 384 honk300-status-goose.svg -resize 36x36 -colorspace sRGB
  -depth 8 PNG32:honk300-status-goose@2x.png`, then verify RGBA, 36×36 pixels, and 18-point AppKit
  template behavior on a real Mac.
- A tray/menu implementation must reuse the shared Configure and graceful-Quit behavior described
  by ADR 0028; the icon does not introduce a second settings model or an abrupt process exit.
