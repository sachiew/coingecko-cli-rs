# Phase 1 Review (`phase-1/foundation`)

Compared `main...phase-1/foundation` (`7a179f5`).

## Findings

### 1) Medium: TUI can render invalid axis label (`Day NaN`) for single-point chart data
- In chart rendering, midpoint label now computes `x_mid / x_max` and formats as float text.
- If `data.len() == 1`, then `x_max == 0.0`, so division is `0.0 / 0.0` and label becomes `Day NaN`.
- This is a behavioral regression in an edge case and violates the Phase 1 "no behavioral changes" goal.

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/src/tui.rs:627`

### 2) Medium: New `CLAUDE.md` contains incorrect architecture/feature statements
- It claims "`--json` flag outputs machine-readable JSON" but this is not implemented in current branch.
- It also says "All API functions return `Result<(), ...>`", which is false (`fetch_*` APIs return typed values).
- This creates misleading project guidance and increases onboarding/debug friction.

References:
- `/Users/hongzhekhooi/coingecko-cli-rs/CLAUDE.md:34`
- `/Users/hongzhekhooi/coingecko-cli-rs/CLAUDE.md:36`

## Verification Notes
- `cargo build` passes.
- `cargo clippy -- -D warnings` passes.
- `cargo test` passes (0 tests).
- Cargo still emits a duplicate-target warning because both `cg` and `coingecko` point to `src/main.rs`; this appears pre-existing and is not introduced by this phase.
