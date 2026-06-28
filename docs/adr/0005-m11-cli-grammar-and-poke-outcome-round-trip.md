# ADR 0005: M11 Three-Name CLI, Goose-Speak, and Poke-Outcome Round-Trip

- **Status:** Accepted
- **Date:** 2026-06-28
- **Milestone:** M11

## Context

M10 established the local control plane (`STOP` / `RELOAD` / `DO <action>`) over a per-user
single-instance channel, but it answered every command with `OK` at the moment the command was
*enqueued* in the server thread — before the simulation had processed it. M11 adds the
user-facing CLI grammar on top: three invocation names (`honk300`, `honk`, `goose`), goose-speak
normalization (`plz` → start, `bad` / `no` / `no honk` → stop), and `do <action>` pokes that map
to engine `PokeAction`s.

Both clients — the root CLI and the M12 config TUI — were written to surface a real per-command
outcome (`Ok` → "accepted", `Err(code)` → "rejected: code"). The transport never delivered one:
`honk300 do nab` reported success even when the engine rejected the action as `Busy` or
`Unsupported`, and the TUI's "poke rejected" branch was dead code.

M11 and M12 were implemented in a single session and landed as one commit without an ADR. An
adversarial review against `honk300_plan.md` surfaced both the missing-ADR process gap and the
always-OK control response. This ADR records the as-built M11 contract, with the response path
corrected.

## Decision

The three-name grammar and goose-speak normalization live in the root binary's argument
normalization. `honk-engine` never sees an invocation name or a goose-speak phrase; it only
receives the closed `PokeAction` enum and `WorldOptions`.

`DO` and `RELOAD` now complete a request/response **round-trip**. The transport hands the decoded
command to the simulation paired with a one-shot response channel and blocks — bounded at two
seconds — for the simulation's real result before answering the client:

- `PokeOutcome::Applied` → `OK`
- `PokeOutcome::Busy` → `ERR BUSY`
- `PokeOutcome::Unsupported` → `ERR UNSUPPORTED`
- reload success → `OK`; reload rejected (config failed to load) → `ERR RELOAD_REJECTED`
- `STOP` → `OK`, then the instance exits

The wire response stays the finite `OK` / `ERR <code>` protocol from ADR 0004 (≤128-byte frames).
No new response variants are introduced; engine outcomes map onto existing codes.

## Consequences

- `honk300 do <action>` reports the true result. The config TUI's "poke rejected: {code}" status
  now actually fires when the goose declines an action.
- The transport waits on the simulation, capped at two seconds; a frozen or absent simulation
  yields `ERR TIMEOUT` rather than a false `OK`.
- Shutdown stays deadlock-free: on teardown the server observes its shutdown flag and breaks
  *before* reading the self-sent reload sentinel, so it never blocks waiting on a torn-down
  simulation.
- The change is transport-only; the two clients were already written for a real outcome, so no
  client code changed.

## Verification

- `honk-control` unit tests: `PokeOutcome` → `ControlResponse` mapping; the `ControlRequest`
  hand-off/`respond` handshake delivers the answer to the waiting transport; existing protocol
  round-trip, malformed, oversized, and unknown-command tests.
- CLI tests: goose-speak start/stop normalization across all three names, explicit pokes stay
  explicit, lifecycle/config commands parse.
- Functional smoke over the real named pipe: `do honk` → accepted; `do nab` with mouse steal
  disabled → rejected `UNSUPPORTED`; `do meme` → accepted; `stop` → accepted and the instance
  exits.
