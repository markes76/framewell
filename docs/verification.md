# Verification — 12 October 2026

## Passed

- Native Apple Silicon release build and development bundle signature verification.
- Host: `cargo fmt --check`, eleven meaningful document/rendering/export regression tests, `cargo clippy --locked --all-targets -- -D warnings`.
- Guest macOS 27.0.1: self-check performs save/open and PNG rendering; app launched beside the Swift prototype.
- UI: draw an arrow, save the project, relaunch, reopen the project and observe the same editable arrow.
- UI: assemble a guide step from the current rendered image.
- Live AI: synthetic image → Codex caption JSON → draft visible in app → explicit Use as guide caption → project save. Initial CLI variadic-image parsing failure was repaired by separating the positional prompt with `--`.
- Caption adapter runs with read-only sandbox, ephemeral sessions and isolated CLI configuration, without loading the user's unrelated MCP servers. Account authentication remains in Codex; Framewell never reads account tokens.
- Flattened privacy rendering (including blur sampling from protected pixels), save/reopen fidelity, immutable shared source/undo, invalid geometry, unsupported version, asset path traversal, off-canvas annotation handling and HTML caption escaping are regression-tested.

## Pending and limitations

- Native region capture is implemented; its live test reached the macOS Screen Recording gate. The user must authenticate in System Settings directly in the VM. Successful region/window/display captures remain unverified until that permission is granted.
- Guides are currently session-local. Saved projects preserve captions; durable guide storage is a next release gate.
- Existing Swift documents and library are untouched and not migrated yet.
- Current build uses development signing; signing/notarization and automatic permission retention across rebuilds are not production-ready.
- Full manipulation, crop/resize, scrolling, stamps, OCR, JEV decisions and OpenAI pixel generation/editing remain roadmap items. Hebrew/RTL shaping, accessibility and other operating systems have not been verified.
- Only the independent new source tree is intended for public publication. No private screenshots, original project library, commercial binaries, decompiled code or credentials are included.

## Snagit/REA evidence

Installed Snagit: 2026.4.2, build2026.108, bundlecom.TechSmith.Snagit. REA Info.plist Evidence: `ev_240f750971976adf961b84920026b657724f725394a1caa435d9df945a5e4303`. Mach-O Evidence: `ev_7ce2ac7474b5390b4cc3f635d5f80e9fb29f7d288e7ccfe17a0644d3f47e9926`.

Reviewed the installed editor, recent captures, styles/properties, annotation tool menu, capture popover and Create menu. Binary linkage corroborates distinct recording, storage, computer-vision, AI and sharing components; it does not establish their runtime implementation.

Ghidra12.1.4 and OpenJDK21 installed; matching arm64 decompiler confirmed. REA decompiled an original test function correctly. Snagit's full main-binary analysis exceeded the initial330-second deadline; the longer retry completed successfully. REA indexed 47,171 procedures, 93,332 defined strings and 66 segments. Native overview Evidence: `ev_2219b7607906db21336cb79a50d8932a779f35aeb210a22a97b2806e7b28137d`. The inventory is not a feature-by-feature runtime proof or recovery of original source. A private analysis snapshot is retained outside this repository.
