# Contributing

Start with an issue describing the user workflow and acceptance test. Keep platform adapters independent of document logic. Run `cargo fmt --check`, `cargo test --locked` and `cargo clippy --locked --all-targets -- -D warnings`.

Add meaningful regression tests for rendering/privacy, state changes, persistence and async results. Distinguish shipped behavior from proposals. Do not add proprietary reference assets or decompiled code. Never commit credentials or personal capture libraries.
