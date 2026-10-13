# Capture and annotation release gates

The first alpha was an engineering foundation. The 13 October workspace rebuild improves navigation, canvas space, tool visibility, Quick Styles, capture previews and guides. It does not close the full gap to Snagit.

| Workflow | Current Rust prototype | Remaining work |
| --- | --- | --- |
| Capture | macOS region/window/display; guided downward scrolling of a selected area | Automatic/horizontal scrolling, capture presets, delay, cursor options and global shortcut |
| Editor organization | Labeled tools, central canvas, visual styles, inspector tabs, recent captures | Keyboard access and native accessibility audit, richer tool preferences |
| Annotation | Arrow, rectangle, text, rectangular callout, click-placed numbered/lettered steps with editable size/shape/colors and saved step styles, pen, highlight, blur, opaque redaction | Resize handles, multiple selection, snapping, more shapes, callout tails, fill/outline separation, stamps and saved styles for other annotation tools |
| Image manipulation | Fit-relative zoom; PNG/JPEG import and flattened PNG export | Pan, actual-scale zoom, crop, resize, rotation, effects, clipboard workflows and more export formats |
| Library | Local editable projects, titles, thumbnail previews and title search | Tags, favorites, folders, trash/recovery, sorting, bulk operations and legacy Swift import. Paged browsing and search keep older captures accessible |
| Guides | One persistent guide, editable captions, snapshots, reorder/remove, HTML+PNG export | Multiple named guides, templates, richer headings, PDF export and reusable layouts |
| AI | Existing Codex image explanation with explicit draft review | JEV decisions, OCR-backed smart annotations, OpenAI image generation/editing, usage tracking and provider preferences |
| Video | Deferred by user decision | Capture/recording foundation before timeline editing |
| Distribution | MIT source, macOS development bundle and CI | Apple Developer ID signing, notarization, updater and Windows/Linux capture. VM development certificate continuity is verified; ad hoc builds still change identity |

Next acceptance should exercise a real workflow: capture software, crop it, add and resize an explanation, redact private details, save/reopen, assemble a multi-step guide, review AI output, then export. Every gate needs working behavior and evidence, not an inert button.
