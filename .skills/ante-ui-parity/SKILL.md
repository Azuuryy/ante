---
name: ante-ui-parity
description: Compare Ante's Bevy screens with a Balatro capture taken in the same state and at the same resolution, and adjust them.
---

Use the user's capture or the local reference for the phase, filed as described in [the capture library](../../docs/CAPTURES.md) (`captures/reference/`, outside Git). Capture Ante with F6 in the Dev build, at the same size. Read the [layout and acceptance](../../docs/UI_BEVY_GUIDE.md) conventions. First record the client area bounds, anchors, region proportions, spacing, palette and control states. Keep scaling uniform and test the hitboxes after a resize.

Build from Ante's components. Load the available local sprites and check their fallback rendering. The user's requirements take precedence over the reference, notably buttons that darken without moving. Compare side by side at equal size, then check a wide and a narrow window. Report the gaps found and name the states still to compare. A successful build validates the code; a visual comparison needs a capture of the rendering.
