# Progress Bar

## Overview

Progress bars show download completion. They appear in two contexts: inline in table rows and in the detail panel file list. Both use the same visual system.

---

## Visual Spec

| Property      | Value                              |
|---------------|------------------------------------|
| Height        | 3px                                |
| Border radius | 99px (full round)                  |
| Track color   | `var(--muted)`                     |
| Overflow      | hidden                             |

---

## Fill Colors

The fill color and glow depend on the torrent state:

| State       | Fill Color       | Glow Shadow                              |
|-------------|------------------|------------------------------------------|
| Downloading | `var(--accent)`  | `0 0 6px var(--accent-glow)`             |
| Seeding     | `var(--green)`   | `0 0 6px var(--green-glow)`              |
| Completed   | `var(--green)`   | none (static, no glow)                   |
| Paused      | `var(--fg-3)`    | none                                     |
| Errored     | `var(--red)`     | `0 0 6px var(--red-glow)`                |
| Checking    | `var(--amber)`   | none                                     |

Glow is only applied when the torrent is actively transferring. Static bars (completed, paused) do not glow.

---

## CSS Utility Classes

```css
.progress-glow-accent {
  box-shadow: 0 0 6px var(--accent-glow);
}

.progress-glow-green {
  box-shadow: 0 0 6px var(--green-glow);
}
```

---

## Implementation

### Table Row Progress Bar

```jsx
<div style={{
  display: "flex",
  alignItems: "center",
  gap: 8,
}}>
  {/* Track */}
  <div style={{
    flex: 1,
    height: 3,
    borderRadius: 99,
    background: "var(--muted)",
    overflow: "hidden",
  }}>
    {/* Fill */}
    <div
      className={isActive ? "progress-glow-accent" : undefined}
      style={{
        width: `${progress * 100}%`,
        height: "100%",
        borderRadius: 99,
        background: fillColor,
        transition: "width 300ms ease",
      }}
    />
  </div>
  {/* Percentage */}
  <span style={{
    fontSize: 10,
    fontFamily: "'JetBrains Mono', monospace",
    fontVariantNumeric: "tabular-nums",
    color: "var(--fg-2)",
    minWidth: 32,
    textAlign: "right",
  }}>
    {(progress * 100).toFixed(1)}%
  </span>
</div>
```

### Detail Panel File Progress

Same structure but slightly taller (4px height) and without the percentage label (the size text is shown separately).

---

## Width Transition

The fill width transitions smoothly when progress updates:

```css
transition: width 300ms ease;
```

This is one of the few elements where a width transition is used instead of transform. The 3px height makes layout cost negligible.

---

## Do and Don't

**Do:**
- Use `width` percentage for fill (not transform). At 3px height, the layout cost is near zero.
- Show glow only on actively transferring torrents. Remove glow on pause or completion.
- Use `tabular-nums` on the percentage label to prevent jiggling.
- Round the percentage to one decimal place.

**Don't:**
- Use heights above 4px. Thick progress bars dominate visually in dense tables.
- Animate the glow. It is a static box-shadow, not pulsing.
- Show a progress bar for torrents with unknown total size (0 bytes). Show "--" instead.
- Use gradient fills on progress bars. Solid colors are clearer.
