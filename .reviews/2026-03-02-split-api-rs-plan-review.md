# Review: `.plans/split-api-rs.md`

## Findings

1. **High: Cross-module visibility is incomplete and will break compile after split**
   - The plan says to make `MarketCoin` and `ChartData` structs `pub(crate)`, but sibling module `tui_data.rs` currently reads their fields directly.
   - In Rust, sibling modules cannot read private fields even if the struct is `pub(crate)`.
   - Either make accessed fields `pub(crate)` or provide accessors/conversion methods.

2. **Medium: Plan omits preserving `clippy` allow attributes while requiring `-D warnings`**
   - Verification requires `cargo clippy -- -D warnings`.
   - Current `src/api.rs` uses targeted `#[allow(...)]` attributes on extracted functions (`run_trending`, `run_markets`, `display_chart`, `run_history`).
   - If those attributes are dropped during extraction, verification can fail.

3. **Low: Test count note appears stale and can confuse execution checks**
   - Plan mentions date module size as “7 date functions + 25 tests”, while current date tests in `src/api.rs` are 21.
   - `cargo test --all-targets -- --list` currently shows duplicated unit-test listings because there are two bin targets (`cg` and `coingecko`), which can be mistaken for extra tests.

## Suggested plan amendments

- Step 6 (`markets.rs`): clarify field visibility strategy for `MarketCoin` consumed by `tui_data.rs`.
- Step 8 (`history.rs`): clarify field visibility strategy for `ChartData.prices` consumed by `tui_data.rs`.
- Add an explicit extraction note: preserve function-level lint attributes when moving code.
- Update verification wording or counts so reviewers know expected test output shape.
