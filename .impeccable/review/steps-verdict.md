# Step rebuild and guide artwork finish review

Disposition: **ship** — verdict pass: both listed material fixes resolved in the reviewed macOS environment.

Scope: the native macOS Step-tool rebuild and generated guide artwork in Framewell Rust. This is not a review or approval of Snagit parity. Independent source-and-render review; no HTML/CSS detector applies. All five supplied screenshots were opened and are valid captures of their named states. The reported intermittent garbled OpenGL frame is separate adverse evidence and is not erased by those valid captures.

The initial full review below is retained as history. Its two findings are superseded by the verdict pass at the end of this file; the historical corruption description does not describe the tested replacement renderer.

## persistence

The implementation has durable substance: marker shape, fill, label and border are stored on editable objects; named styles are written to `step-styles.json`; the next value is derived from remaining matching labels after history changes. Source supports centered click placement and a next-marker preview. The build packet reports successful live placement, Undo sequence restoration, selected-marker editing, reopen and PNG export; those are reported execution results, not independently replayed in this review. Artwork has recorded origin and a prompt sidecar. Existing prototype boundaries remain honest.

## fidelity

The direction is visibly preserved: a canvas-led editor, labeled vector tools, compact white controls, teal selection and actual image thumbnails. The step inspector supplies the requested sequence, shape and styling controls, and the selected marker is visibly editable. Generated guide artwork is original and complementary to the real guide screenshots. No approved UI comp exists, so fidelity is judged against the code-led contract, not invented pixel targets.

The minimum screenshot does not meet the contract's distinction between discoverable main controls and scrollable advanced saved styles: the first view ends at Quick Styles, with Shape & size nearly entirely below the fold. At desktop, even the Border row is clipped. The settings the user specifically missed should not appear absent again when opening the tool.

## ceiling

This is a meaningful, coherent prototype improvement, with a credible one-click step mechanism. It does not establish whole-product superiority or production readiness. The known full-surface corruption after repeated resizing is a release-blocking reliability gap for this reviewed environment until resolved or conclusively isolated; a two-pixel manual resize is diagnostic information, not an acceptable user recovery. The guide artwork fits the inherited visual world; a wholesale visual rebuild is not warranted.

## material_fixes

1. **P1 — Resolve or isolate repeated-resize rendering corruption.** The packet reports that resizing to minimum a second time and scrolling replaced the entire OpenGL content with horizontal garbled lines, recovered by a roughly two-pixel width change. Reproduce the reported desktop → minimum → desktop → minimum sequence with inspector scrolling, capture the failure if it returns, and establish a renderer/window fix or a supported environment where the problem does not occur. Do not call it a VM-only issue without evidence. Verification must include repeated resize and scroll cycles after the correction, with readable content and working inputs throughout; document any remaining environment restriction explicitly.

2. **P2 — Keep primary Step settings visible at the supported minimum size.** Compact or reorganize the Step inspector so the initial placement view at 1100 × 700 inner points exposes Sequence/Next value, Shape, Size, Text and fill/border controls without scrolling through promotional copy and large style samples first. Saved styles and batch actions may remain below the fold as the contract allows. Preserve readable text and adequate targets; use compact rows or a collapsed preset group rather than uniformly shrinking everything. Recheck both placement and selected-marker states at desktop and minimum. The minimum capture must show where the core settings are, rather than stopping above them.

## keep

Keep centered one-click placement, clear Next marker feedback, numeric and letter sequences, white-bordered bold marker labels, editable original objects, the four visual preset samples, the shared teal/neutral desktop language, and the honest synthetic content labels. Keep the original guide artwork and its restrained placement; no new visual identity is needed. Carry the final compact-inspector behavior into design documentation after fixes, since current DESIGN.md still describes the previous Step text-before-styles model and generic button treatment.


## Verdict pass — 13 October 2026

**Disposition: ship**, scoped to the two material fixes from the full review. This is a fix-list verdict, not a new whole-surface or whole-product approval.

All six final captures were opened: `steps-desktop.png`, `steps-minimum.png`, `steps-edit.png`, `steps-edit-minimum.png`, `guide-art-desktop.png`, and `guide-art-minimum.png`. Each is valid and readable, with no garbled regions. Source confirms the macOS Wgpu renderer and target-specific Metal dependency.

| Finding | Score | Evidence and boundary |
| --- | --- | --- |
| P1 — repeated-resize corruption | Resolved | macOS replaces the failing OpenGL rendering path with Wgpu/Metal. The build packet reports three clean small-window transitions, inspector down/up scrolling, selected-marker interaction, and Guides scrolling in the installed signed build. Final captures corroborate intact rendering. Runtime interaction results were reported by the build agent, not replayed by this reviewer. The original OpenGL cause remains unproven; resolution applies to this replacement renderer in the tested environment, without a claim about other machines. |
| P2 — primary settings below fold | Resolved | The inspector extends to the status bar while the filmstrip remains below the canvas. Both minimum-size placement and selected-marker captures show Shape, Size, Text, fill/text/border colors and border width without scrolling. Sequence/Next value is visible in placement and editable Label is visible in selection. Advanced actions can scroll as allowed. Readable type and the current-marker selection affordance remain intact. |

Persistence and keep: the core placement/editing model and original artwork are retained. Fidelity: the minimum-size settings contract is now visibly met. Ceiling: stop the correction loop; both listed fixes are resolved. Carry final panel geometry and renderer verification limits into documentation. No additional defect hunt was conducted in this verdict pass.
