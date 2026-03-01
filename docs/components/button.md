# Button

## Variants

Tsubasa has two button variants. No other variants should be created.

### Primary (`.btn-primary`)

The highest-emphasis action on screen. Gradient background, white text, glow on hover.

```jsx
<button className="btn-primary">
  <Plus size={13} />
  Add Torrent
</button>
```

**Visual spec:**

| Property       | Value                                                                  |
|----------------|------------------------------------------------------------------------|
| Background     | `var(--gradient-accent)` (135deg diagonal)                             |
| Text color     | `#fff`                                                                 |
| Font size      | 12px                                                                   |
| Font weight    | 500                                                                    |
| Padding        | 6px 12px                                                               |
| Border radius  | `var(--radius-md)` (6px)                                               |
| Border         | none                                                                   |
| Box shadow     | `0 1px 2px rgba(0,0,0,0.3), 0 0 0 1px rgba(255,255,255,0.08) inset`  |
| Hover shadow   | `0 2px 8px var(--accent-glow), 0 0 0 1px rgba(255,255,255,0.12) inset`|
| Active         | `translateY(1px)`, `opacity: 0.85`                                     |

**States:**

```
default  -> gradient bg, white text, subtle inner glow
hover    -> opacity 0.92, outer glow appears
active   -> translateY(1px), opacity 0.85
disabled -> opacity 0.5, cursor: not-allowed
```

### Ghost (`.btn-ghost`)

Secondary actions. Transparent by default, subtle fill on hover.

```jsx
<button className="btn-ghost">
  <Settings size={13} />
  Settings
</button>
```

**Visual spec:**

| Property       | Value                           |
|----------------|---------------------------------|
| Background     | `var(--overlay)`                |
| Text color     | `var(--fg-2)`                   |
| Font size      | 12px                            |
| Padding        | 6px 10px                        |
| Border radius  | `var(--radius-md)` (6px)        |
| Border         | `1px solid var(--line)`         |
| Hover bg       | `var(--muted)`                  |
| Hover text     | `var(--fg)`                     |
| Hover border   | `var(--line-strong)`            |

---

## Icon-Only Buttons

Used in the toolbar and modal close buttons. Must have `title` or `aria-label`.

```jsx
<button
  className="transition-colors-fast"
  title="Close"
  style={{
    width: 30, height: 30,
    borderRadius: "var(--radius-md)",
    border: "none",
    background: "transparent",
    cursor: "pointer",
    color: "var(--fg-3)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  }}
>
  <X size={15} />
</button>
```

Hover: `background: var(--muted)`, `color: var(--fg-2)`.

---

## Rules

1. Only one primary button per visible area. If two actions compete, one must be ghost.
2. Button labels are sentence case ("Add torrent", not "ADD TORRENT").
3. Icon size in buttons: 13px. Always left of text.
4. Gap between icon and text: 6px (from the flex `gap` property).
5. Disabled buttons use `opacity: 0.5` and `pointer-events: none`. Never gray out the text color separately.
6. Loading state: replace the icon with `<Loader2 size={12} style={{ animation: "spin 1s linear infinite" }} />`. Keep the label text.

---

## Do and Don't

**Do:**
- Use `.btn-primary` for the single most important action (Add Torrent, Save Settings).
- Use `.btn-ghost` for everything else (Browse, Pause All, Cancel).
- Add `transition-colors-fast` class to icon-only buttons.

**Don't:**
- Create a "danger" button variant. Use a ghost button with `color: var(--red)` on hover for destructive actions (see context menu).
- Nest a button inside another button.
- Use `<a>` styled as a button. Use `<button>` with an `onClick` handler.
- Apply `box-shadow` to ghost buttons. Only primary buttons get shadows.
