# Capsule Definition of Done (v0)

A capsule is considered done for CAELES v0 when all items below pass.

## 1. Build

- `caeles build <capsule-path>` succeeds.
- Output wasm exists at `target/wasm32-unknown-unknown/.../*.wasm`.

## 2. Run

- `caeles run --manifest <manifest>` or `--capsule-id` succeeds.
- Runtime returns a `run-<timestamp>` id.
- Run status is `exited`.

## 3. Package

- `caeles package ...` succeeds.
- Package directory contains:
  - `manifest.json`
  - `capsule.wasm`
  - `package.json`

## 4. Pull

- `caeles pull <capsule-id>` succeeds.
- Pulled directory contains:
  - `manifest.json`
  - `capsule.wasm`

## 5. Inspect

- `caeles inspect <capsule-id>` returns valid capsule metadata.
- `manifest_exists` is `true`.
- Last run metadata is visible after at least one run.

## 6. Logs

- `caeles logs <run-id>` returns non-empty output.
- Includes start line and run exit status line.

## 7. Manifest Contract (v0)

- Required fields: `id`, `name`, `version`, `entry`, `permissions`, `lifecycle`.
- `lifecycle.kind` must be `on_demand`.
- Unknown fields are rejected.

## 8. Permissions Contract (v0)

- `notifications=false` blocks notifications at runtime.
- `network=false` blocks host-mediated network access (`host_http_get`).

## 9. Regression Safety

- Unit + integration tests pass locally and in CI.
- CI checks pass: format, clippy (warnings as errors), tests.
