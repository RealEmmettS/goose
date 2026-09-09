# ADR 0053: Recognize atomic GNOME consent replacement within the existing deadline

Date: 2026-09-09

Status: Implemented; native qualification in progress.

## Evidence

During the combined desktop update, standalone GNOME run 34328703661 at
`97856fa55688794997384040cfa5c566b2c85b09` failed its Debian ARM64 external-revocation
check. The saved status reported failed observations and the runtime logged an unavailable
response during the initial consent read. Its trace confirmed no remaining observations,
pointer state, drag target or riding task. The complete saved record remained readable.

The private reader first checks pathname metadata and then opens a stream. An atomic
replacement between those operations gives the stream a different inode. The old reader
rejects that race as an unavailable observation before reading the complete revocation.
A controlled native-I/O fixture reproduces the exact unavailable-versus-revoked result
against the production JavaScript. It fails before the correction and passes afterward.
The hosted diagnostic does not retain private record contents or expose them in logs.

## Decision

For the mutable consent record only, close the mismatched stream and repeat its full
pathname/opened-stream validation once. Use the same cancellable and original request
deadline. A second replacement remains a failed observation. Ownership, private modes,
file type and size must pass before an inode/device replacement is eligible for this retry.
Companion script and metadata replacements retain their existing immediate rejection.

Read and validate the resulting complete record normally: a revoking phase or changed
nonce withdraws consent, malformed content fails, and active consent still requires all
existing companion, caller and second-read checks before any observation is returned.
No permission, backend, transport, worker-recovery or observation capability changes.

## Verification

The production-code fixture checks a rename during open, a second replacement, malformed
replacement, an equivalent active record, unsafe ownership/mode changes and a stalled
retry. It verifies no observations on failure, bounded attempts, stream cleanup, retained
request admission and the original cancellation deadline. Existing partial-record,
companion-tampering, actor bounds and shutdown tests remain unchanged.

Require fresh qualification on both supported GNOME Shell versions and architectures,
plus the complete unchanged-source release gates recorded in
[v1.11.0 readiness](../readiness/v1.11.0-readiness.md).
