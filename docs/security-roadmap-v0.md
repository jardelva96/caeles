# Security Roadmap (v0 -> v1)

This roadmap defines the hardening path for capsule isolation and host capability control.

## Current State (v0)

- WebAssembly runtime execution through wasmtime.
- Explicit host ABI (`caeles`) for capabilities.
- Permission enforcement implemented for:
  - notifications (`host_notify`)
  - network (`host_http_get`)

## Gaps

- No per-capsule network allowlist/denylist yet.
- No per-run quota (timeouts, memory ceilings, request limits) yet.
- No signed package verification yet.
- Runtime logs are host-driven and should evolve to structured event logs.

## Next Hardening Steps

## 1. Network policy controls

- Add `permissions.network_policy` in manifest:
  - allowed hosts
  - allowed schemes
  - optional port constraints
- Deny by default when policy is absent.

## 2. Runtime quotas and limits

- Add execution timeout per run.
- Add max linear memory pages.
- Add host call quotas (e.g., network calls/run).

## 3. Capability-scoped ABI

- Move from broad functions to capability handles issued at startup.
- Reject calls without valid capability token.

## 4. Artifact integrity

- Add digest in package metadata.
- Add optional signature verification on package load.

## 5. Structured auditing

- Emit JSONL security events:
  - permission_denied
  - network_request
  - network_failure
  - runtime_trap

## 6. Android host controls

- Enforce app-level network security config.
- Route runtime events into Android logging/telemetry.
- Apply policy updates from host app settings.
