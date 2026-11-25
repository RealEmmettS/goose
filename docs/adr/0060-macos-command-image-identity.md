# 0060 — Locate the running Mac app from command aliases

## Decision

Use Darwin’s process image path for ownership lookups and sibling launches. All three user command symlinks can reach the same signed app. Use its existing receipt, root, signatures, hashes and lifecycle checks. Other platforms retain their native Rust path lookup.

## Evidence

Fresh public v1.11.3 and v1.11.4 reported an unknown origin through the command alias and a managed shell origin through the bundle executable on the same hosted account. The older bundle command can perform the upgrade. Opening the DMG again or running the website installer also repairs an older installation. Candidate and public qualification must exercise the command aliases after both installation routes on Apple Silicon and Intel.
