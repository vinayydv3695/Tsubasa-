# Table

## Overview

The torrent table is the primary content area. It uses `@tanstack/react-table` for sortable columns and renders a virtualized list of torrent rows.

---

## Structure

```
+----------------------------------------------------------+
| Header row (sticky, uppercase labels, 10px, weight 600)  |
+----------------------------------------------------------+
| Row 1  [Name] [Size] [Progress] [Status] [Speed] [ETA]  |
| Row 2  ...                                                |
| ...                                                       |
+----------------------------------------------------------+
```

---

## Visual Spec

### Header

| Property        | Value                              |
|-----------------|------------------------------------|
| Background      | transparent                        |
| Text color      | `var(--fg-3)`                      |
| Font size       | 10px                               |
| Font weight     | 600                                |
| Text transform  | uppercase                          |
| Letter spacing  | 0.5px                              |
| Padding         | 8px 12px                           |
| Border bottom   | `1px solid var(--line)`            |
| Cursor          | pointer (for sortable columns)     |

Sort indicator: `ChevronUp` or `ChevronDown` icon (10px) next to the active column header, colored `var(--accent)`.

### Rows

| Property          | Value                                |
|-------------------|--------------------------------------|
| Height            | 34px                                 |
| Padding           | 0 12px                               |
| Background        | transparent                          |
| Hover background  | `var(--overlay)`                     |
| Selected bg       | `var(--accent-soft)`                 |
| Border bottom     | `1px solid var(--line-subtle)`       |
| Font size         | 12px                                 |
| Text color        | `var(--fg)` (name), `var(--fg-2)` (secondary columns) |
| Cursor            | pointer                              |
| Transition        | `background-color 100ms ease`        |

### Columns

| Column   | Width     | Alignment | Font             | Notes                          |
|----------|-----------|-----------|------------------|--------------------------------|
| Name     | flex: 1   | left      | Inter 500        | Truncate with ellipsis         |
| Size     | 80px      | right     | JetBrains Mono   | `formatBytes()` output         |
| Progress | 120px     | left      | n/a              | Progress bar + percentage      |
| Status   | 90px      | center    | Inter 500        | Badge pill                     |
| Down     | 80px      | right     | JetBrains Mono   | Green text when > 0            |
| Up       | 80px      | right     | JetBrains Mono   | Accent text when > 0           |
| ETA      | 70px      | right     | JetBrains Mono   | `--fg-3` when idle             |

---

## Progress Bar (Inline)

The inline progress bar inside table rows:

| Property      | Value                                    |
|---------------|------------------------------------------|
| Height        | 3px                                      |
| Border radius | 99px                                     |
| Background    | `var(--muted)` (track)                   |
| Fill          | `var(--accent)` for downloading          |
|               | `var(--green)` for completed/seeding     |
|               | `var(--red)` for errored                 |
| Glow          | `box-shadow: 0 0 6px var(--accent-glow)` when active |

Percentage text: displayed to the right of the bar, 10px monospace, `var(--fg-2)`.

---

## Context Menu

Triggered by right-click on a row.

| Property        | Value                                                |
|-----------------|------------------------------------------------------|
| Background      | `var(--surface)`                                     |
| Border          | `1px solid var(--line-strong)`                       |
| Border radius   | 8px                                                  |
| Box shadow      | `var(--shadow-lg)`                                   |
| Backdrop filter  | `blur(12px) saturate(160%)`                         |
| Padding         | 4px                                                  |
| Min width       | 180px                                                |

### Menu Items

| Property          | Value                           |
|-------------------|---------------------------------|
| Height            | 30px                            |
| Padding           | 0 12px                          |
| Font size         | 12px                            |
| Icon size         | 13px                            |
| Icon-text gap     | 8px                             |
| Border radius     | 6px                             |
| Hover background  | `var(--muted)`                  |
| Danger hover bg   | `var(--red-soft)`               |
| Danger hover text | `var(--red)`                    |

Divider: `1px solid var(--line)` with 4px vertical margin.

---

## Empty State

When no torrents match the filter:

```jsx
<div style={{
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  height: "100%",
  gap: 12,
}}>
  <Download size={32} color="var(--fg-3)" />
  <p style={{ fontSize: 14, fontWeight: 500, color: "var(--fg-2)" }}>
    No torrents yet
  </p>
  <p style={{ fontSize: 12, color: "var(--fg-3)" }}>
    Click the Add button to get started.
  </p>
</div>
```

---

## Do and Don't

**Do:**
- Sort by column on header click. Show direction indicator.
- Highlight the selected row with `var(--accent-soft)` background.
- Show the context menu at the cursor position, clamped to viewport bounds.
- Use monospace for all numeric columns.

**Don't:**
- Add row animations or staggered entrance effects. Tables render instantly.
- Use alternating row colors (zebra striping). The hover + border approach is cleaner.
- Wrap text in table cells. Always truncate with ellipsis.
- Show more than 7 columns. Horizontal scrolling is not supported.
