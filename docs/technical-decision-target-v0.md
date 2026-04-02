# Technical Decision Record - Capsule Target v0

## Decision

For CAELES v0, capsules target `wasm32-unknown-unknown`.

## Date

2026-04-02

## Context

We need a stable capsule target while ABI, permissions, and host integrations are still evolving.

## Rationale

- Runtime capabilities are explicit through host ABI imports.
- Avoids accidental dependency on WASI host surface during v0 hardening.
- Fits current runtime implementation and sample capsules.
- Reduces complexity for first Android host PoC.

## Consequences

- Capsules should rely on CAELES host ABI for runtime services.
- If/when WASI support is introduced, it should be an explicit v1 decision with policy and sandbox controls.

## Follow-up

- Revisit target strategy after:
  - runtime policy controls (network allowlists, quotas)
  - structured audit events
  - Android host bridge stabilization
