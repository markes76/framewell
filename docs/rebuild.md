# Framewell Rust rebuild

User-approved scope, 12 October 2026: capture, annotation, visual guides and AI first; a full video timeline later. Keep Framewell as the product name. Build an original, open-source implementation. The Swift prototype and its library remain independent.

## Architecture

- Rust document engine: immutable shared pixel source, serializable editable objects, bounded undo history, document revisions.
- Rust raster renderer: canvas and PNG exports share one rendering implementation. Privacy masks render last and opaque, regardless of annotation order.
- Disk persistence: immutable source PNG assets and an atomically replaced JSON manifest. The old Swift `.framewell` format is not silently interpreted as the new format.
- Desktop shell: egui/eframe with a labeled tool row, local library, canvas, properties and guide workspace. Original graphics and open fonts.
- Platform adapters: macOS interactive capture via the OS capture service, isolated from document code. Other platforms initially support import/edit/export.
- AI jobs: background requests, snapshot image, captured document revision, explicit draft review, no automatic modification or sharing. The initial adapter uses the user's existing Codex CLI account; direct OpenAI image generation and JEV layout review are migration work, not completed capabilities.

## Review of the installed Snagit

Snagit 2026.4.2 (build 2026.108) is installed and open in the macOS VM. Observed: image/video-specific editor controls; labeled annotation tools; independent Quick Styles and Tool Properties; recent-image strip; library; capture popover with image/video/all-in-one, cursor, delay, clipboard and editor toggles; Create menu with Image from Template and Video from Images. These observations inform functional requirements, not copied code or assets.

REA verified native ARM64 identity and Info.plist metadata. Its Mach-O inspection reported Apple imaging/text/media frameworks and separate vendor storage, recorder, AI and sharing dependencies. Dependency names establish linkage, not how those features execute. The initial deep-analysis calls were unavailable because no engine was configured. Engine installation and a focused native investigation are ongoing separately. Proprietary binary files and decompiled output are excluded from this public source repository.

## Initial acceptance gates

1. Build and launch on Apple Silicon macOS 15+ in the VM.
2. Import a PNG/JPEG, annotate with editable objects, move/select/delete, undo/redo, save/reopen, export matching pixels.
3. Region/window/display capture, cancellation and permission errors remain explicit.
4. Redaction obscures sensitive pixels in flattened exports, even if other annotations overlap it. Editable projects still contain their source image and must not be described as sanitized exports.
5. Guide steps have editable captions and safe HTML+PNG export.
6. AI explanations use explicit requests and review; a changed document rejects stale results. Network output, keys and account tokens are not stored in the project repository.

## Work after the initial build

Native capture presets/cursor/delay, robust window/display targeting, direct manipulation handles and layer ordering, crop/resize, stamps, source effects, OCR adapters, scrolling capture, durable multi-step guides, library metadata/Trash, legacy import validation, JEV layout decisions, OpenAI image generation/editing, usage ledger, Hebrew/RTL shaping, accessibility, installers, stable signing/notarization and cross-platform capture. Video recording and timeline editing follow the agreed priority.

Do not claim Snagit parity, production readiness or a complete rewrite while these gates remain open.
