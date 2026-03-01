# Dashboard Layout

## Overview

The dashboard is the main application layout. All content is visible at once in a single-window, fixed-viewport interface. There are no routes or page transitions. Panels resize, collapse, and appear/disappear in place.

---

## Layout Structure

```
+------------------------------------------------------------------+
| Toolbar (44px, full width, data-tauri-drag-region)                |
| [+ Add] [Open] [Search] [Pause] [Resume]   [DL speed] [UL speed]|
+----------+-------------------------------------------------------+
| Sidebar  | Torrent Table (flex: 1)                                |
| 208px    |                                                       |
| or 48px  |  [Name]  [Size] [Progress] [Status] [Down] [Up] [ETA] |
| (col-    |  Row 1                                                 |
| lapsed)  |  Row 2                                                 |
|          |  ...                                                   |
|          +-------------------------------------------------------+
|          | Detail Panel (resizable, min 200px)                    |
|          |  [General] [Files] [Peers] [Trackers] [Cloud]          |
|          |  Tab content area                                      |
+----------+-------------------------------------------------------+
| Status Bar (28px, full width)                                     |
| [status dot] [active count]    [total count] [DL] [UL] [cloud]  |
+------------------------------------------------------------------+
```

---

## CSS Layout (Flex-Based)

```jsx
<div style={{ display: "flex", flexDirection: "column", height: "100vh", overflow: "hidden" }}>
  {/* Toolbar */}
  <Toolbar />  {/* height: 44px, flexShrink: 0 */}

  {/* Middle section */}
  <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
    {/* Sidebar */}
    <Sidebar />  {/* width: 208px | 48px, flexShrink: 0 */}

    {/* Content area */}
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Torrent table */}
      <TorrentTable />  {/* flex: 1, overflow: auto */}

      {/* Detail panel (shown when a torrent is selected) */}
      <DetailPanel />   {/* flexShrink: 0, height: user-defined or 40% */}
    </div>
  </div>

  {/* Status bar */}
  <StatusBar />  {/* height: 28px, flexShrink: 0 */}
</div>
```

---

## Fixed Elements

These elements have fixed heights and do not resize:

| Element    | Height | Behavior      |
|------------|--------|---------------|
| Toolbar    | 44px   | Always visible|
| Status bar | 28px   | Always visible|

---

## Flexible Elements

These elements adapt to available space:

| Element      | Behavior                                                         |
|--------------|------------------------------------------------------------------|
| Sidebar      | Fixed width (208px or 48px). Full height between toolbar and status bar. Internal scroll on overflow. |
| Torrent table| `flex: 1`. Takes all remaining vertical space. Internal scroll for rows. Sticky header. |
| Detail panel | Visible when a torrent is selected. Height is min 200px, default 40% of content area. |

---

## Overflow Strategy

- **Window**: `overflow: hidden` on `html`, `body`, and `#root`. No page-level scrolling.
- **Sidebar**: `overflowY: auto`, `overflowX: hidden`. Scrollbar is 5px, styled with theme colors.
- **Torrent table**: `overflowY: auto`. Sticky header row. Scrollbar is 5px.
- **Detail panel**: `overflowY: auto` on the tab content area. Tabs themselves do not scroll.
- **Settings modal**: `overflowY: auto` on the content area. Tab sidebar does not scroll.

---

## Z-Index Scale

| Layer           | z-index | Elements                           |
|-----------------|---------|-------------------------------------|
| Base content    | 0       | Sidebar, table, detail panel        |
| Context menu    | 30      | Table right-click menu              |
| Modal backdrop  | 50      | Settings overlay                    |
| Modal content   | 51      | Settings card                       |
| Toast container | 60      | Bottom-right toast stack            |

---

## Tauri Window Integration

### Drag Region

The toolbar is the window drag region:

```jsx
<div data-tauri-drag-region style={{ height: 44, ... }}>
  {/* Non-draggable interactive children */}
  <button>Add</button>  {/* -webkit-app-region: no-drag */}
</div>
```

Interactive children (buttons, inputs, links) are excluded from drag via the CSS rule:

```css
[data-tauri-drag-region] button,
[data-tauri-drag-region] input,
[data-tauri-drag-region] a {
  -webkit-app-region: no-drag;
}
```

### Window Dimensions

Default window size defined in `tauri.conf.json`:
- Width: 1100px
- Height: 700px
- Min width: 800px
- Min height: 500px

---

## Responsive Behavior

Tsubasa is a desktop-only application. There are no breakpoints, no media queries, and no mobile layout. However, the interface gracefully handles window resizing:

| Window Width | Behavior                                                  |
|--------------|-----------------------------------------------------------|
| > 1100px     | Full sidebar (208px) + all columns visible                |
| 800-1100px   | Auto-collapse sidebar to 48px. All columns visible.       |
| < 800px      | Below minimum width. Tauri enforces minimum.              |

The sidebar collapse is triggered by the user via the toggle button or automatically when the window is narrow (handled by `useUIStore`).

---

## Do and Don't

**Do:**
- Use `flex` for all layout. No CSS Grid at the top level.
- Set `overflow: hidden` on all layout containers to prevent scrollbar cascading.
- Keep the toolbar and status bar fixed height. They are structural anchors.
- Use `flexShrink: 0` on fixed-height elements.

**Don't:**
- Use `position: absolute` for layout panels. Use flex flow.
- Add horizontal scrolling to any panel.
- Allow the torrent table to shrink below 200px height.
- Create sub-pages or routes. Everything is single-window.
