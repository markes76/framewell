# Framewell Rust

Framewell is an original open-source macOS capture, annotation, visual guide, and AI workspace. The Rust rebuild is an early prototype, not a Snagit replacement yet. The user explicitly chose capture, annotation, guides, and AI first, with video timeline editing later.

The user rejected the first alpha's cramped controls and filename library. The current design brief explicitly asks for Snagit's familiar editor organization, visible features, and usable tools. Use original implementation and Framewell branding. Preserve editable annotations, undo, opaque redaction, local project files, PNG export, and review-before-apply AI. Never claim feature parity from a UI resemblance.

Platform: macOS desktop, Rust / eframe. Scene: colleagues explaining software at a desk. Documents stay local; an explicit AI request sends the rendered image through the existing Codex login. JEV and OpenAI image generation integration remain planned for this rebuild. Preserve the separate Swift app and its library.

Priority: large canvas, labeled annotation tools, visual Quick Styles, contextual properties, actual capture thumbnails, and a dedicated guide workspace. Missing scrolling capture, OCR, richer effects, stamps, and image generation must remain accurately documented.
