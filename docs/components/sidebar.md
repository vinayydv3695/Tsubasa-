# Sidebar

## Overview

The sidebar is the primary navigation surface. It provides filtering by status, category, tag, and tracker via collapsible sections modeled after qBittorrent.

---

## Structure

```
+--- Sidebar (aside) ---+
| Brand Header           |
|   [Logo] Tsubasa [<]   |
|------------------------|
| STATUS (section)       |
|   All Torrents    [12] |
|   Downloading      [3] |
|   Seeding           [5] |
|   Completed         [2] |
|   Paused            [1] |
|   Error             [0] |
|   Cloud             [1] |
|------------------------|
| CATEGORIES (section)   |
|   Movies            [4] |
|   TV Shows          [2] |
|   Music             [1] |
|   Games             [3] |
|   Software          [0] |
|   Other             [2] |
|------------------------|
| TAGS (section)         |
|   (Untagged)        [8] |
|   Important         [4] |
|   [+ Add tag...]        |
|------------------------|
| TRACKERS (section)     |
|   tracker.xyz       [6] |
+------------------------+
```

---

## Dimensions

| State     | Width | Transition                              |
|-----------|-------|-----------------------------------------|
| Expanded  | 208px | `width 200ms cubic-bezier(0.4,0,0.2,1)` |
| Collapsed | 48px  | Same                                     |

---

## Brand Header

| Property      | Value                                    |
|---------------|------------------------------------------|
| Height        | 44px                                     |
| Border bottom | `1px solid var(--line)`                  |
| Child: Logo   | 22x22, radius 6px, `var(--gradient-accent)`, `box-shadow: 0 0 8px var(--accent-glow)` |
| Child: Name   | 13px, weight 600, letter-spacing -0.3px  |
| Child: Toggle | `ChevronLeft` (14px), ghost style        |

When collapsed, only the logo badge is shown, centered.

---

## Section Headers

| Property        | Value                                  |
|-----------------|----------------------------------------|
| Font size       | 9px                                    |
| Font weight     | 700                                    |
| Text transform  | uppercase                              |
| Letter spacing  | 0.8px                                  |
| Color           | `var(--fg-3)`                          |
| Padding         | 6px 8px 4px 10px                       |
| Margin top      | 8px                                    |
| Toggle icon     | `ChevronDown` (collapsed) / `ChevronUp` (expanded), 10px |

Hidden when sidebar is collapsed.

---

## Navigation Items

| Property           | Value                                       |
|--------------------|---------------------------------------------|
| Height             | ~32px (7px top/bottom padding)              |
| Padding (expanded) | 7px 8px 7px 10px (indent: 22px left)        |
| Padding (collapsed)| 7px 0, centered                             |
| Font size          | 12px                                        |
| Icon size          | 13px                                        |
| Icon-text gap      | 7px                                         |
| Transition         | `background 100ms ease, color 100ms ease`   |

### States

| State    | Background             | Text Color       | Other                     |
|----------|------------------------|------------------|---------------------------|
| Default  | transparent            | `var(--fg-2)`    |                           |
| Hover    | `var(--muted)`         | `var(--fg)`      |                           |
| Active   | `var(--accent-soft)`   | `var(--accent)`  | Left accent bar (3px)     |

### Active Left Accent Bar

```jsx
<span style={{
  position: "absolute",
  left: 0, top: "50%",
  transform: "translateY(-50%)",
  width: 3, height: 14,
  background: "var(--accent)",
  borderRadius: "0 2px 2px 0",
  boxShadow: "0 0 6px var(--accent-glow)",
}} />
```

### Count Badge

| Property      | Value                                 |
|---------------|---------------------------------------|
| Font size     | 10px                                  |
| Font family   | JetBrains Mono                        |
| Font variant  | tabular-nums                          |
| Background    | `var(--muted)` (default), `var(--accent-soft)` (active) |
| Color         | `var(--fg-3)` (default), `var(--accent)` (active)       |
| Padding       | 0px 5px                               |
| Border radius | `--radius-full`                       |
| Min width     | 18px                                  |
| Text align    | center                                |

---

## Category System

Categories are auto-detected from torrent names using regex heuristics (defined in `src/stores/categories.ts`):

| Category  | Detection Pattern                     | Icon     | Color         |
|-----------|---------------------------------------|----------|---------------|
| Movies    | 1080p, BluRay, x264, etc. (no S01E02) | Film     | `var(--red)`  |
| TV Shows  | S01E02, "season", "episode"           | Tv       | `var(--blue)` |
| Music     | FLAC, MP3, 320kbps, album, OST        | Music    | `var(--green)`|
| Games     | repack, fitgirl, GOG, CODEX           | Gamepad2 | `var(--amber)`|
| Software  | crack, keygen, portable, installer     | Monitor  | `var(--accent)`|
| Other     | No pattern match                       | FolderOpen | `var(--fg-3)` |

---

## Tag System

Tags are user-created string labels stored in `localStorage` via `src/stores/categories.ts`:

- Tags appear in the TAGS section.
- Each tag shows a count of assigned torrents.
- Tags can be created inline via the "Add tag..." button with a text input.
- Tags can be removed with an X button that appears on hover.
- The special "(Untagged)" entry shows torrents with no assigned tag.

---

## Collapsed Behavior

When collapsed (48px width):
- Only STATUS section nav items are shown (icon-only, centered).
- Section headers, labels, and count badges are hidden.
- CATEGORIES, TAGS, and TRACKERS sections are hidden entirely.
- A `ChevronRight` toggle button is shown at the bottom.
- Nav items get `title` attribute for tooltip on hover.

---

## Do and Don't

**Do:**
- Show live counts next to each filter item.
- Collapse sections independently (each has its own boolean state).
- Use `title` attributes on collapsed nav items for accessibility.
- Persist sidebar collapse state in `useUIStore`.

**Don't:**
- Add more than 4 sections. More sections would overflow on small screens.
- Use horizontal scrolling in the sidebar.
- Show empty categories (categories with 0 torrents can remain visible but dimmed).
- Put action buttons in the sidebar. It is for navigation only.
