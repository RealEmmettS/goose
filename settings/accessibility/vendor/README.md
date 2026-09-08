# AccessKit AT-SPI state correction

`accesskit_atspi_common/` contains the published `accesskit_atspi_common` 0.19.1
source from crates.io, whose archive SHA-256 is
`023da0e5097f46df7092d5280b02efb9bbf8d93298daeced42652463e357d636`.
Its recorded upstream commit is
`c88605b96d04431f9c3c792464a0f2f253480e94`, path `platforms/atspi-common`.
The original normalized manifest, source manifest, README and VCS identity remain.
Registry cache markers, the upstream changelog and its independent lockfile are
omitted. Licensing texts are the same pinned AccessKit/Chromium texts retained in
`../licenses/`; generated package notices include this local dependency.

The production correction in `src/node.rs` adds `!state.is_disabled()` to the
branch that sets AT-SPI Enabled/Sensitive. Upstream's read-only role check excludes
buttons, so disabled buttons otherwise advertise those states despite having no
Action interface. The existing read-only mapping is preserved. The only other
source addition registers `honk_disabled_state_tests.rs`, which exercises the
actual consumer-to-AT-SPI state translation for enabled/disabled controls and
read-only text. Native D-Bus fixtures additionally verify the disabled Update now
button and its subsequent update-check-driven availability.

This crate is a member of the bridge's isolated workspace, so `cargo test
--workspace --locked --manifest-path settings/accessibility/Cargo.toml` executes
the regression alongside bridge tests. Remove this patch when a qualified pinned
upstream release provides equivalent behavior, keeping the native regression.
