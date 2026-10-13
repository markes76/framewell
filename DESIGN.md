---
name: Framewell Rust
description: Native macOS capture, annotation, visual library, and guide workspace.
colors:
  accent: "#1b696f"
  selection-tint: "#e0f0f0"
  ink: "#252c34"
  muted: "#55606b"
  border: "#dae0e5"
  canvas: "#e7ebef"
  white: "#ffffff"
  panel: "#f9fafb"
  recent-surface: "#f6f8fa"
  workspace-surface: "#f3f6f8"
  tool-hover: "#efefef"
  annotation-blue: "#2269bd"
  annotation-red: "#c73b42"
  annotation-purple: "#9b52b8"
  annotation-amber: "#d1831a"
  annotation-green: "#3c874f"
  annotation-brown: "#935a39"
  step-teal: "#176f76"
  step-coral: "#cf523d"
  step-cobalt: "#335bba"
  step-ink: "#273141"
  guide-cover: "#0a363f"
  guide-cover-copy: "#cce4e6"
typography:
  step-marker:
    fontFamily: "Ubuntu"
    fontSize: "30px"
    fontWeight: 700
  guide-cover-title:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "30pt"
  body:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "14pt"
  button:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "14pt"
  small:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "12pt"
  tool-label:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "13pt"
  section:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "15pt"
  guide-step:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "18pt"
  brand:
    fontFamily: "macOS UI (runtime SFNS), egui proportional fallback"
    fontSize: "22pt"
rounded:
  control: "6pt"
  guide-cover: "10pt"
  quick-style: "5pt"
  image-tile: "5pt"
  tool: "6pt"
  guide-card: "6pt"
spacing:
  item: "8pt"
  button-x: "12pt"
  button-y: "8pt"
  panel-x: "16pt"
  inspector: "14pt"
  workspace: "24pt"
  guide-card: "18pt"
  quick-style-gap: "7pt"
  library-x: "16pt"
  library-y: "18pt"
components:
  button-primary:
    rounded: "{rounded.control}"
    backgroundColor: "{colors.accent}"
    textColor: "{colors.white}"
    typography: "{typography.button}"
    padding: "8pt 12pt"
    height: "34pt"
    width: "92pt"
  button-secondary:
    rounded: "{rounded.control}"
    backgroundColor: "{colors.white}"
    textColor: "{colors.ink}"
    typography: "{typography.button}"
    padding: "8pt 12pt"
  button-secondary-hover:
    backgroundColor: "{colors.selection-tint}"
    textColor: "{colors.ink}"
  workspace-tab:
    rounded: "{rounded.control}"
    typography: "{typography.button}"
    height: "34pt"
    width: "72pt"
  workspace-tab-active:
    backgroundColor: "{colors.selection-tint}"
    textColor: "{colors.ink}"
  tool:
    textColor: "{colors.ink}"
    typography: "{typography.tool-label}"
    rounded: "{rounded.tool}"
    width: "67pt"
    height: "66pt"
  tool-active:
    backgroundColor: "{colors.selection-tint}"
    textColor: "{colors.accent}"
  quick-style:
    backgroundColor: "{colors.white}"
    rounded: "{rounded.quick-style}"
    width: "59pt"
    height: "50pt"
  recent-image:
    backgroundColor: "{colors.white}"
    rounded: "{rounded.image-tile}"
    typography: "{typography.small}"
    width: "136pt"
    height: "92pt"
  library-image:
    backgroundColor: "{colors.white}"
    rounded: "{rounded.image-tile}"
    typography: "{typography.small}"
    width: "220pt"
    height: "176pt"
  step-preset:
    width: "56pt"
    height: "54pt"
    rounded: "{rounded.control}"
  guide-cover:
    backgroundColor: "{colors.guide-cover}"
    textColor: "{colors.white}"
    rounded: "{rounded.guide-cover}"
    height: "160pt"
  guide-card:
    backgroundColor: "{colors.white}"
    rounded: "{rounded.guide-card}"
    padding: "18pt"
---

# Design System: Framewell Rust

## Overview

**Creative North Star: "User-pinned Snagit-style desktop workspace"**

Framewell uses the user-pinned Snagit-style desktop workspace: a large image canvas, labeled vector tools, visual styles, contextual properties, and a visual capture library. Original Framewell branding and code define the implementation.

Editor, Library, and Guides share light desktop surfaces, readable dark text, muted supporting copy, teal selection, and compact controls. Content previews carry the visual hierarchy; color highlights actions and selection rather than decorating the workspace. This is a native macOS Rust/egui prototype, with no claim of Snagit feature parity.

**Key Characteristics:**
- Canvas-led desktop hierarchy with visible labeled tools.
- Shared teal selection, neutral surfaces, and image previews across workspaces.
- Editable annotations and contextual properties, with reviewed AI captions.

## Colors

The primary palette combines muted teal with light cool neutrals. The YAML frontmatter owns the exact values.

### Primary

- **Accent teal:** Capture action, authored vector tools in their selected state, selection outlines, and Framewell branding.
- **Selection tint:** Current workspace, active tools, selected thumbnail cards, and Quick Styles. General egui hovered and active widget backgrounds also use this tint.

### Secondary

- **Annotation presets:** Blue, red, purple, amber, green, and brown accompany teal and ink in the eight-color Quick Styles grid. These style image annotations; they are not competing navigation accents. Step has its own four named content presets: Teal and Coral circles, a Cobalt square, and an Ink diamond. The guide cover adds a deep teal artwork surface and pale supporting copy.

### Neutral

- **Ink and muted:** Main text/vector strokes and supporting copy respectively.
- **White and border:** Main header, card and style interiors, and restrained one-point borders.
- **Canvas:** The field around the centered fitted image.
- **Panel, recent surface, and workspace surface:** Tool/inspector/status panels, recent strip/title region, and Library/Guides respectively.
- **Tool hover:** The custom tool button's neutral hover or focus fill when inactive.

**The Shared Selection Rule.** Use teal and its pale tint for the current workspace, selected tool, selected image, and active visual style. Annotation preset colors remain content colors.

The initializer uses a nearby teal for the initial annotation color. Per-frame workspace selection overrides that initializer with the shared tint and accent stroke; do not create a second navigation accent from the initialization value. The selected-annotation canvas outline is separately blue and is a manipulation affordance.

## Typography

The app reads `/System/Library/Fonts/SFNS.ttf` at runtime, inserts it as `macOS UI` into egui's proportional family, and retains egui's default proportional fallback if loading fails. No Apple fonts are bundled or redistributed.

Body and buttons share the main reading size; small labels identify thumbnails and status. Tool labels use the intermediary size, section titles use a modest stronger size, guide steps use a larger strong label, and the Framewell brand anchors the shared header. The guide cover has a separate, larger title role. Exact sizes are in the frontmatter. `.strong()` is an egui emphasis request, not an extracted numeric font weight. Generic `ui.heading()` and native egui widget details remain framework defaults rather than invented typography tokens.

New step markers use the bundled open Ubuntu Bold font in preset/cursor previews and raster export. Their default text size is recorded as image pixels, with automatic reduction to fit the marker; legacy steps retain their existing font behavior. All native UI measurements here use egui logical points, independent of screenshot pixel density. Step size, text size, and border width controls explicitly use image pixels. The sidecar's HTML previews map one logical point to one CSS pixel for representative panel previews; they are translations of native components, not app implementation code.

## Layout

The normal inner viewport is (1380 × 850 points), with a minimum of (1100 × 700 points). Native window chrome is outside these inner dimensions. No mobile breakpoint or compact alternate workspace is implemented.

The shared top header uses horizontal margins (16 points) and vertical margins (10 points), with equally aligned controls (34 points). Workspace tabs have a minimum width (72 points); Capture has a minimum width (92 points). The Region/Window/Display/Scrolling selector participates in the same header interaction height and has a configured width (88 points).

The Editor adds a shelf of tool controls (66 points tall each), with top/bottom shelf padding (5 points) and gaps (4 points). The right inspector is fixed and non-resizable (286 points), with internal margins (14 points), tabs, and an always-visible vertical scrollbar. The central image is fitted and centered in the remaining canvas. The recent strip is fixed (142 points), horizontally scrollable, and contains the current capture followed by up to 24 recent saved captures. It spans only the canvas region; the inspector extends beside it down to the shared footer. Status and fit/zoom controls sit in that footer. Step properties use compact spacing so sequence or label, four presets, shape, size, text size, and fill/text/border controls remain visible at the minimum viewport; saved styles and later actions may require scrolling.

Library and Guides use the shared header and status bar with workspace margins (24 points). Library cards form a responsive column count based on available width divided by (236 points), with grid gaps (16 horizontal / 18 vertical points). Search matches titles across the entire loaded library; pages contain at most 24 entries. Unloaded previews remain labeled, clickable entries rather than making older captures inaccessible.

Guide cards contain a preview column (300 points), an image fitted within (300 × 190 points), a gap (20 points), and a caption/action column with a minimum width (200 points). Cards are separated by (14 points). The Guides workspace starts with a full-width artwork banner (160 points high), a right-hand image area (240 points wide), and left-aligned title and supporting copy. These native desktop layouts shrink within the supported minimum viewport; they do not become a web or phone layout. The exported HTML guide is separately responsive: its content container caps at 980 CSS pixels and, at a maximum width of 650 CSS pixels, hides decorative cover art and reduces heading size and padding. That export breakpoint does not apply to the native editor.

**The Equal Header Controls Rule.** Keep the workspace tabs, Import, Save, Export, capture-mode selector, and Capture aligned to the shared header control height.

## Elevation & Depth

The authored workspace is flat. Panel fills, white cards, restrained borders, and tint selection separate surfaces without custom drop shadows or animated lifting. No authored shadow or motion scale exists. Native egui popup, focus, disabled, and window treatments remain framework behavior; do not invent project tokens for them.

## Shapes

Tools and guide cards use gently curved corners; Quick Styles and image tiles use slightly tighter corners. The frontmatter records their actual radii. Quick Style and thumbnail cards use a border (1 point), strengthening selected borders to (2 points) in accent teal. Custom tool focus uses an accent outline (1 point). Tool symbols are authored vector strokes, usually (1.8 points) on the shelf, within a (28 × 28 point) icon area. Reuse those coherent vector forms across surfaces. Shared inactive, hovered, and active widget visuals explicitly use six-point corners. Inactive weak button fills are white; hovered and active fills use selection tint. Native framework field and popup details otherwise remain framework behavior. Step content shapes are circles, sharp-cornered squares, or diamonds; do not round the square marker to match its control.

## Components

### Buttons and navigation

Capture is the filled teal primary action with white text. Import, Save, Export, and the workspace tabs use egui buttons with the shared typography, padding, and header control height. Selected tabs use the common selection treatment. The capture selector exposes Region, Window, Display, and Scrolling. Inspector navigation uses selectable labels for Properties, Layers, and AI. Native framework hover/focus and disabled responses apply except where custom controls explicitly paint their own state.

### Capture permission and guided scrolling

Capture checks macOS screen-recording permission before hiding the editor. Missing permission presents recovery guidance in the status area: enable the installed Framewell Rust app in Privacy & Security → Screen & System Audio Recording, then quit and reopen once. Permission denial, capture failure, and user cancellation are distinct outcomes; cancellation does not present a permission error or replace the current image.

Scrolling is guided manual downward capture on the primary display. Its native egui modal places a display preview beneath a heading and two short instruction lines. Drag inside the preview to select scrolling content, excluding fixed headers, borders, and scrollbars. A square teal outline marks the selection; the selected pixel dimensions appear below it. Start scrolling capture remains disabled until both selected dimensions reach (32 pixels). Cancel, Escape, and a backdrop click dismiss the preview through the modal close response. The editor's Escape action is guarded while this preview is open.

The preview fits within the viewport with an available width capped at (900 points) and height based on the viewport minus (230 points), with a (220-point) floor. These are local component constraints, not new system tokens. The minimum-size evidence shows the instructions, preview, disabled Start, and Cancel together inside the modal; its dimmed backdrop protects the editor beneath it.

Starting hides the editor and opens a non-resizable, always-on-top native control window (380 × 120 points). It shows a frame count, progress or recovery copy, Finish, and Cancel. Placement prefers the bottom-left then bottom-right display corner, with top corners as fallbacks, and keeps the controls outside the selected area. If none fits, the preview asks for a smaller selection. The user scrolls the real window slowly, retaining at least 20% overlap; frames are sampled with an (850-millisecond) pause between capture iterations. Finish returns the stitched image to Editor with a prompt to review its joins before sharing. Cancel, closing the controls, or capture failure retains the prior image.

This bounded extension stops at 80 accepted frames or 120 seconds; output is limited to 40 million pixels and 16,000 pixels per dimension. It does not establish automatic scrolling, horizontal scrolling, sticky-content handling, other display configurations, or Snagit parity. The existing palette, type hierarchy, compact native controls, and canvas-led editor remain its visual authority.

### Labeled vector tools

Each tool pairs a vector icon with a readable label and a descriptive tooltip. Move, Arrow, Text, Callout, Shape, Step, Pen, Highlight, Blur, and Redact occupy the same control size. Active tools tint the tile and recolor the icon and label; inactive hovered/focused tools use the neutral hover fill. Focus additionally draws an accent outline. Undo and Redo sit together beside the shelf.

### Quick Styles and properties

For non-Step tools, eight visual presets form a four-column grid. Each preview uses the current tool's vector symbol and a preset color; later presets use a heavier stroke or larger text size. Selecting a style changes the selected annotation in place when present. Properties include relevant text, color, and width controls, plus a guide caption field.

**The Text Before Styles Rule.** For Text and Callout, show the editable text field before Quick Styles; the inspector scrolls when the remaining properties exceed its height.

### Step markers

Step placement is one click at the marker center, with a next-marker preview. Its compact inspector starts with numeric, uppercase, or lowercase sequence choices, next value, and Restart. Editing through Move replaces sequence controls with an editable Label and Continue placing steps. Both modes then show four visual presets followed by shape and size controls. Size ranges from 24–160 image pixels, text from 10–80, and border width from 0–12; fill, text, and border colors are independent. The default marker is a 56-pixel circle with white text and a two-pixel white border. Applying a style preserves an existing marker's center and label. Apply style to all steps and the collapsible My saved styles follow the primary settings. Named styles persist locally in `step-styles.json`, with up to twelve retained presets.

**The Step Settings Rule.** Keep sequence or selected-marker label, four presets, and shape/size/color controls ahead of saved-style management; allow the inspector to scroll for later actions.

### Inputs and AI review

The library has a single-line title search field with a configured width (260 points). Editor titles use an unframed field with configured width (300 points). Annotation text and guide captions use multiline fields. The AI inspector collects a focus prompt, displays a working state, and presents an editable draft before the user chooses to use it as a guide caption. This does not imply image generation or automatic application.

### Image tiles and guide cards

Recent and Library tiles use the same thumbnail-and-caption form at their frontmatter sizes. Images preserve their aspect ratio and fit inside an inset image box. Titles clip inside the card and remain available in a hover tooltip. Active tiles use the shared tint and stronger accent border. Guide cards use white interiors and padding, numbered step labels, editable captions, and Move up, Move down, and Remove actions. Guide images are rendered snapshots of the image and its annotations; they are not editable annotation canvases inside the guide. Original local artwork from `assets/framewell-guide-cover.png` supplements those screenshots in the native banner and is copied as `cover.png` into the HTML export. The export uses its own larger cover and section radii and system typography; its decorative artwork disappears on narrow screens while step images retain their aspect ratio. Artwork is not a substitute for actual guide screenshots.

## Do's and Don'ts

### Do:
- Do keep header workspace tabs, action buttons, and the capture-mode selector aligned at the shared header control height.
- Do use authored vector tool icons with visible labels and tooltips.
- Do put editable Text and Callout text before Quick Styles; use the dedicated sequence-or-label ordering for Step.
- Do retain real image thumbnails, title search across the full library, and pagination.
- Do load the installed macOS font at runtime and retain the egui proportional fallback.
- Do share the palette and spacing grammar across Editor, Library, and Guides.
- Do keep primary Step settings visible at the minimum viewport and saved styles below them.
- Do preserve actual screenshots beneath the decorative guide cover.
- Do keep scrolling selection protected by the native modal, place Finish/Cancel outside the selected content, and return the result to the editor for join review.
- Do distinguish capture permission recovery from cancellation and capture failure; retain the prior image on cancellation or failure.

### Don't:
- Don't replace the visual library with a cramped filename sidebar.
- Don't redistribute Apple font files.
- Don't describe automatic or horizontal scrolling, OCR, crop/resize handles, stamps, richer effects, JEV, OpenAI image generation/editing, or video timeline editing as shipped features.
- Don't infer production signing or notarization from the VM's persistent local development certificate; public development bundles remain self-signed and unnotarized.
- Don't treat resemblance to the reference as evidence of feature parity.

Evidence: `src/widgets.rs` defines the palette, vector controls, style previews, tile dimensions and state painting; `src/main.rs` defines runtime fonts, shared egui style, canvas treatment and viewport limits; `src/workspace.rs` defines headers, panel dimensions, full-library filtering/pagination, properties, AI review, and guide cards. `.impeccable/review/editor-desktop.png` and `editor-minimum.png` confirm the editor composition and minimum-width Callout-text placement. `PRODUCT.md` and `docs/editor-surface.md` establish the user-pinned reference and truthful prototype boundaries. Missing features are release gaps, not new visual rules.

Capture-extension evidence: `src/scroll_ui.rs` defines the native preview modal, modal close handling, selection outline, local sizing and external Finish/Cancel controls; `src/platform.rs` distinguishes permission, cancellation and capture failure; `src/main.rs` guards editor Escape during the preview and preserves the previous image on cancelled/failed scrolling; `src/scrolling.rs` bounds the stitched output. `.impeccable/review/scrolling-result.png` confirms the normal editor, Scrolling selector, stitched synthetic content and unchanged-image cancellation status. Private `scroll-minimum.png` and `scroll-controls.png` under `/Users/mbp-private/Downloads/framewell-capture-review/` were inspected for minimum-size modal fit and controls outside the content; they remain outside the public repository. `docs/verification.md` records VM capture and cancellation tests, signing continuity and remaining coverage limits. The screenshots corroborate composition, not every cancellation path or production readiness. The scrolling extension introduced no new system tokens; the later Step and guide-artwork refresh adds only the built values documented above.

Step/artwork refresh evidence: `src/step_ui.rs` defines compact placement/edit properties and saved styles; `src/step.rs` defines marker geometry and Ubuntu Bold raster rendering; `src/main.rs` registers the bold font, six-point widget corners, panel order, and macOS WGPU renderer; `src/workspace.rs` defines the native artwork banner; `src/lib.rs` defines responsive HTML export. `.impeccable/review/steps-desktop.png`, `steps-minimum.png`, `steps-edit-minimum.png`, and `guide-art-desktop.png` corroborate the final composition. The macOS build uses Metal through WGPU to address the observed resize corruption; the finish review resolved the minimum-size settings and observed resize issues within this iteration. This is scoped evidence, not whole-application parity or exhaustive renderer coverage.
