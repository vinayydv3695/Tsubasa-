# Design System

## Design Philosophy

Tsubasa is a power-user desktop application that should feel like a professional tool, not a toy. Every design decision follows three principles:

1. **Density without clutter.** Show a lot of information in a small space, but use whitespace, borders, and hierarchy to prevent visual overload.
2. **Calm confidence.** Dark backgrounds, muted secondary text, and subtle borders create a stable foundation. Accents and glows are used sparingly to draw attention only where it matters.
3. **Instant feedback.** Every user action produces visible feedback within 120ms. No loading state should exist without an indicator. No click should go unacknowledged.

---

## Visual Principles

### Layered Surfaces

The interface uses a strict surface hierarchy. Each layer is a step lighter than the one beneath it. Components never skip layers.

| Token        | Role                                  | Example                     |
|--------------|---------------------------------------|-----------------------------|
| `--base`     | Window background                     | App background, body        |
| `--surface`  | Primary content panels                | Sidebar, settings modal     |
| `--overlay`  | Cards, dropdowns, inputs              | Input fields, context menus |
| `--muted`    | Hover states, disabled backgrounds    | Button hover, scrollbar     |
| `--subtle`   | Pressed states, secondary highlights  | Active pressed states       |

**Rule:** A child element must sit on a surface equal to or one step above its parent. Never place `--overlay` directly on `--base` without a `--surface` container in between.

### Accent Economy

The accent color is the most valuable visual signal. It communicates "this is interactive" or "this is active." Overusing it devalues its meaning.

- Use accent for: active navigation, focused inputs, primary buttons, progress indicators, the one most important element on screen.
- Do not use accent for: decorative elements, static labels, large background areas, borders that separate content.

### Border Strategy

Borders separate content zones. Three intensities exist:

| Token            | Opacity | Use Case                                      |
|------------------|---------|-----------------------------------------------|
| `--line-subtle`  | 3-4%    | Row separators inside tables, divider lines   |
| `--line`         | 6-7%    | Panel borders, input borders, section dividers|
| `--line-strong`  | 10-12%  | Modal borders, hover-state borders, emphasis  |

**Rule:** Use `--line` as the default. Escalate to `--line-strong` only for containers that float above the page (modals, popovers, context menus).

---

## Component Consistency Rules

### Sizing

| Element                | Height | Padding (horizontal) | Font Size | Border Radius   |
|------------------------|--------|---------------------|-----------|-----------------|
| Primary button         | 30px   | 12px                | 12px      | `--radius-md`   |
| Ghost button           | 30px   | 10px                | 12px      | `--radius-md`   |
| Icon-only button       | 28px   | 6px                 | n/a       | `--radius-md`   |
| Text input             | 32px   | 12px                | 12px      | `--radius-lg`   |
| Badge                  | 18px   | 6px                 | 10px      | `--radius-full` |
| Sidebar nav item       | 32px   | 10px                | 12px      | 6px             |
| Table row              | 34px   | 12px                | 12px      | 0               |
| Tab button             | 32px   | 12px                | 12px      | 0               |

### Icon Usage

- Icons: Lucide React, stroke width 1.5 (default), size 13px in buttons and nav, 12px in badges, 15px in headers.
- Always pair icons with text labels in navigation. Icon-only buttons are acceptable only in toolbars with tooltips.
- Icon color follows text color. Active icons use `--accent`.

### Spacing Within Components

- Gap between icon and label: 6px (buttons), 7px (nav items), 4px (badges).
- Internal padding: 6px vertical, 10-12px horizontal for interactive elements.
- Section title bottom margin: 12px. Section gap: 24px.

### State Consistency

Every interactive element must express these states:

| State    | Visual Rules                                                           |
|----------|------------------------------------------------------------------------|
| Default  | Standard colors, standard border                                       |
| Hover    | Background lightens one step, text brightens, border may strengthen    |
| Active   | `translateY(1px)` or opacity reduction, pressed feel                   |
| Focused  | `0 0 0 3px var(--accent-soft)` ring + `border-color: var(--accent)`   |
| Disabled | `opacity: 0.5`, `cursor: not-allowed`                                 |
| Selected | `background: var(--accent-soft)`, `color: var(--accent)`, accent bar  |

---

## Do and Don't

**Do:**
- Use `var(--fg-2)` for secondary text, `var(--fg-3)` for tertiary/hint text.
- Use monospace font (`JetBrains Mono`) for all numeric data: speeds, sizes, percentages, timestamps.
- Use the badge utility classes for status labels.
- Use `transition-colors-fast` class for elements that respond to hover.

**Don't:**
- Hardcode color values. Always reference CSS variables.
- Use more than one primary button per visible area. If two actions compete, one is primary, the other is ghost.
- Add shadows to flat elements inside panels. Shadows are only for floating layers (modals, popovers, toasts).
- Use animation durations longer than 300ms for UI transitions. Desktop users expect snaps, not slides.
