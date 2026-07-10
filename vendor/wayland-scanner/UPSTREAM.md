# wayland-scanner security backport

This directory is the crates.io `wayland-scanner` 0.31.10 source, licensed under the included
MIT license. Project changes are the `quick-xml` dependency update from 0.39 to 0.41 and the
corresponding upstream `xml10_content` call required by the newer parser API.

That dependency change is the fix from upstream wayland-rs revision
`d07c4f91f28b42e5a485823ffd9d8d5a210b1053` ("wayland-scanner: Bump quick-xml dependency to
0.41"). The full git revision also follows unreleased, breaking scanner/client/backend API
changes and cannot be mixed with the released smithay-client-toolkit 0.20 stack. Vendoring the
released scanner surface with the exact d07 dependency fix keeps the application buildable while
removing the vulnerable quick-xml line.

Remove this directory and the root `[patch.crates-io]` entry when a compatible crates.io
wayland-scanner release includes the d07 fix.
