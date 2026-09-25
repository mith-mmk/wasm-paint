# External UI design QA

**Findings**

- No actionable visual P0/P1/P2 differences remain in the desktop comparison. The browser version has no window controls, keeps the canvas at 640 × 480 px, disables unsupported tools, and opens a generated landscape sample; these are documented implementation constraints.
- [P2, fixed] Numeric brush size could display a new value while the range and drawing state retained the old value. After the input handler change, 100 → 37 → 8 updated both controls, and an actual pointer stroke rendered at the selected size 8.
- [P2, fixed] Layer visibility toggled the eye state without changing the composite. The Rust compositor now honors the outer AnimationLayer enable flag while retaining the existing inner frame check. Hiding Layer 1 removed its green stroke and showing it restored the stroke; hiding the main layer removed its red stroke while the Layer 1 stroke remained.
- Eyedropper revision verified: hovering shows sampled HEX, canvas position, and opacity; clicking a sample applies its color and restores the prior Brush tool and size (23). Escape cancels, keeps the prior color (`#EBF3F4`), restores Brush and size 23, and reports cancellation. Arrow/Enter keyboard sampling and transparent-pixel continuation were not verified.
- Save fallback verified: the File menu exposes “PNGをダウンロード…”. Its filename dialog accepts `drawing.v1`; confirmation reports `drawing.v1.png のダウンロードを開始しました。保存完了はブラウザーで確認してください`. Cancel preserves the dirty state and reports cancellation; reopening starts with the default `無題` filename. Ctrl+S showed a saved-byte status, but neither its downloaded file nor dimensions were inspected. The browser download event did not arrive in the harness. Native Save As (`showSaveFilePicker`) could not be operated here and remains unverified.

**Source and implementation evidence**

- Source visual truth: `C:/Users/misir/Downloads/Codex 画像 2026年9月24日 16_43_17.png` (1586 × 992 px).
- Desktop implementation: `C:/temp/temp-paint/ui-qa/sample-home-1586x992.jpg` (1586 × 992 px, CSS viewport 1586 × 992, devicePixelRatio 1). It shows the sample open on the Home tab.
- Mobile implementation: `C:/temp/temp-paint/ui-qa/mobile-390x844.jpg` (390 × 844 px, CSS viewport 390 × 844, devicePixelRatio 1). The sample and status message are visible; the document has no horizontal overflow (375 px scroll width at 390 px viewport).
- Low-height implementation: `C:/temp/temp-paint/ui-qa/desktop-1280x720.jpg` (1280 × 720 px). The document fits the viewport; the right sidebar provides its own scroll area for lower controls.
- The source and desktop implementation were emitted together in one browser QA tool call at their matching native pixel dimensions. No density normalization or scaling was applied. Full-view comparison covered the toolbar, tool rail, canvas, palette, layer panel, and status/zoom bar. Focused comparison was not needed because labels, palette controls, layer rows, icons, and sample art were legible at 1586 × 992.

**Fidelity surfaces**

- Fonts and typography: Japanese labels remain readable with a clear hierarchy. The implementation uses slightly smaller/lighter control labels than the reference, without wrapping or clipping at desktop size.
- Spacing and layout: the ribbon, left tool rail, central canvas, and right sidebar follow the reference grouping. At 1280 × 720 the full application fits; longer sidebar content scrolls inside the sidebar.
- Colors and tokens: active blue selection, neutral panels, palette colors, and disabled gray tools remain consistent with the source.
- Image quality and assets: the generated landscape sample follows the broad sky, cloud, sun, hills, and meadow composition, with details differing from the source. Tool icons use the selected Microsoft icon library.
- Copy and content: labels are coherent in Japanese; fill is available with a tolerance control, while selection, text, and shape actions remain visibly disabled.
- Responsiveness and accessibility: the 390 × 844 capture keeps the sample, tool controls, and status reachable; the status element is displayed and readable. Browser-native keyboard/pointer interactions used in this test worked for the verified controls.

**Interaction coverage**

- Verified: initial transparent 640 × 480 canvas; palette color choice; brush range and numeric size; actual pointer drawing with pencil and brush; erasing a stroke segment; eyedropper hover preview, click sampling, Escape cancel, and prior tool/size/color restoration; adding/selecting layers; layer opacity; per-layer visibility after the compositor fix; sample load from the File menu; zoom in/reset; Color and Canvas tabs; color/layer panel collapse and expand; clear confirmation cancel; fit-to-screen; PNG filename dialog, cancel/dirty retention, clean reopen, and fallback download-start status; 1586 × 992, 1280 × 720, and 390 × 844 layouts.
- Not verified: fill region boundaries, tolerance behavior, and undo/redo; general image file picker path (it shares the sample import path); native Save As picker; actual downloaded PNG file and dimensions; Arrow/Enter eyedropper sampling and transparent-pixel continuation.
- Console: no error or warning entries were recorded in the final desktop sample session.
- Static checks: `node --check tests/web-paint/external-ui.js` and `git diff --check` passed.

**Comparison history**

1. Initial UI testing found numeric size state drift and layer visibility not affecting rendered pixels. The size input now synchronizes the range/state and was rechecked with a size-8 pointer stroke. The compositor now honors the outer AnimationLayer enable flag while retaining the inner frame check; UI checks confirmed hide/show and independent visibility across two layers.
2. Earlier low-height and mobile status layout issues were corrected. The final 1280 × 720 screenshot has no document overflow, and the latest mobile screenshot shows the status message with the sample present.
3. After user feedback, the eyedropper and save fallback received focused browser checks. The filename dialog and fallback start status work; browser download completion/native picker operation remain outside the verified evidence.

**Implementation Checklist**

- [x] Compare the provided 1586 × 992 source screenshot and rendered desktop screen at matching dimensions.
- [x] Verify desktop, low-height, and mobile layouts and primary paint/layer interactions.
- [x] Recheck and clear the numeric brush size and layer visibility P2 findings.
- [x] Recheck the revised eyedropper preview/cancel/restore behavior.
- [x] Recheck Save fallback naming, cancellation, dirty state, and observable download-start status.

**Open Questions**

- Whether the browser-hosted environment exposes the native `showSaveFilePicker` dialog to this test harness. If unavailable, record the UI fallback result and leave the native path explicitly unverified.

final result: passed
test gaps: native Save As picker and downloaded file contents/dimensions unverified
