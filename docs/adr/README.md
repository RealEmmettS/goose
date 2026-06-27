# Architecture Decision Records

This folder records durable architecture decisions for `honk300`. Use it when a change affects platform boundaries, the engine/backend contract, renderer architecture, deployment targets, packaging shape, permissions, or milestone scope.

## Maintenance

- Add a new numbered ADR for each meaningful architectural decision. Use `NNNN-short-title.md`.
- Keep historical ADRs intact. If a decision changes, create a new ADR that supersedes the old one instead of rewriting history.
- Update the task board, `CHANGELOG.md`, `HUMAN_CHANGELOG.md`, `README.md`, `AGENTS.md`, and `CLAUDE.md` when an ADR changes current project guidance.
- Keep `honk-engine` platform-free unless an ADR explicitly changes that rule.

## Index

- [0001 — M7 Cursor Mischief, Renderer Direction, And Cross-Platform Guardrails](./0001-m7-cursor-mischief-renderer-and-platform-guardrails.md)
- [0002 — M8 Foreign-Window Watch-And-Ride Contract](./0002-m8-foreign-window-watch-and-ride.md)
