# ADR 0045: Editable exports from the production vector drawing routine

Status: Accepted, 2026-09-08.

## Context

The refinement uses a procedural vector goose and requires actual renderer exports
for visual review and website artwork. PNG exports preserve the appearance but do
not provide editable paths to the separate website task.

## Decision

The existing projected drawing routine emits its paths through a private, statically
dispatched vector surface. The desktop surface calls the same tiny-skia operations
with unchanged geometry, paint, transform and stroke arguments. The preview exporter
also supplies an SVG surface that serializes those operations in their original order.

SVG output contains editable paths, explicit colors, opacity and transforms. It embeds
no raster images, fonts, executable script or external references. The Rust rig and
projected drawing code remain authoritative; there is no separately maintained SVG rig
or alternate runtime backend. The ordinary renderer gains no recording buffer or
dynamic dispatch. The preview exports each named pose, heading and motion frame.

## Qualification

The existing engine suite and image goldens must remain unchanged. The first exporter
passed all engine tests and image goldens, and all 32 corresponding preview PNGs were
byte-identical to the previously inspected first-stage exports. Independent librsvg
rendering of all 22 named SVGs preserved the intended appearance; expected antialiasing
differences were recorded in the development evidence. Publication and native desktop
acceptance remain separate from artwork export.
