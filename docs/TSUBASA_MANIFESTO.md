# TSUBASA 翼 — Product Manifesto

**Version 2.0 — Foundation Redesign**
Internal Design & Strategy Document

---

## Part 1 — Identity

### Core Philosophy

Tsubasa is a **transfer instrument**. Not a download manager. Not a dashboard. An instrument — like a well-made tool that disappears into the act of using it.

The name means *wing*. Movement without friction. A file should travel from source to disk the way air moves under a wing — effortlessly, with invisible engineering underneath.

**Tsubasa stands for:**

- **Transparency of process.** The user sees exactly what is happening, at exactly the level of detail they choose. No mystery spinners. No ambiguous states. Every byte is accounted for.
- **Intelligent defaults.** The application should work perfectly with zero configuration. Smart Mode routes torrents through cloud or local swarm automatically. The user isn't a network engineer — they're someone who wants a file.
- **Quiet confidence.** The interface doesn't shout about its features. It presents information with the composure of a flight instruments panel — dense when needed, silent when not.

**Tsubasa rejects:**

- Feature theater — adding capabilities just to fill a settings page.
- Visual noise — gradients, shadows, and decorations that exist for their own sake.
- Patronizing UX — confirmation dialogs for reversible actions, tooltips on obvious buttons, tutorial overlays on second launch.
- The assumption that power users want complexity. They want *control*, which is different.

**Tsubasa refuses to become:**

- A media center. It moves files. It does not play, organize, or curate them.
- A social platform. No comments, ratings, or community features.
- A browser. No built-in search beyond torrent provider APIs. No embedded web views.
- Configuration bloat. Every setting must justify its existence against the cost of one more decision the user has to make.

### Emotional Tone

| Axis | Position | Rationale |
|------|----------|-----------|
| Calm ↔ Aggressive | **Calm** | Transfers run in the background. The UI should feel like a quiet control room, not an arcade. |
| Technical ↔ Elegant | **Both, situationally** | The table view is technical — dense data, monospace numbers, zero decoration. The modals and onboarding are elegant — generous spacing, considered typography. |
| Powerful ↔ Minimal | **Powerful surface, minimal chrome** | Every pixel of chrome that doesn't carry information is overhead. But the information itself should be rich. |

**How users should feel:**

- *"This takes my work seriously."*
- *"I trust this to handle large transfers overnight."*
- *"I didn't have to learn anything — it just worked."*
- *"This is clearly built by someone who uses it."*

### Design Principles

1. **Every element earns its pixel.** If it doesn't inform a decision or enable an action, it's deleted. Decorative elements are permitted only when they reduce cognitive load (e.g., a subtle separator that creates scannable groups).

2. **Information density scales with expertise.** Default view shows name, progress, speed, state. Detail panel reveals peers, trackers, file trees. Settings are sectioned by frequency of use. Nothing is hidden — but nothing is forced.

3. **Color is signal, not decoration.** The accent color means "interactive" or "active." Green means "healthy/complete." Amber means "attention." Red means "error." Every other element is neutral grayscale. Violating this mapping destroys the signal system.

4. **Motion proves state change.** Animation exists to confirm that the system responded. A row slides in because a torrent was added. A progress bar fills because bytes are arriving. Motion without semantic meaning is banned.

5. **Surfaces, not shadows.** Depth is communicated through background color stepping, not drop shadows. The only exception: modals and toasts, which float above the document and need a backdrop to break the reading flow.

6. **Text hierarchy is law.** Exactly three foreground levels: `--fg` (primary), `--fg-2` (secondary), `--fg-3` (tertiary). A component that uses all three has clear hierarchy. A component that uses only `--fg` is flat and scannable. A component that uses four levels is broken.

7. **The sidebar is navigation, not workspace.** It shows where you are and where you can go. It does not show data, counters, graphs, or status badges. That is the main panel's job.

8. **Monospace for data, proportional for labels.** Speeds, sizes, hashes, timestamps — monospace. Action labels, descriptions, section titles — proportional. Never mix within a single text run.

9. **One primary action per visible context.** If the user sees two blue buttons, we've failed. Competing primaries create decision paralysis. Secondary actions use ghost or subtle styles.

10. **Ship less, better.** A feature that works flawlessly at launch is worth ten features that need a blog post to explain.

### Non-Goals

**Tsubasa will NOT:**
- Include a built-in media player or file previewer.
- Support plugins, extensions, or user scripts.
- Display advertisements, analytics, or telemetry banners.
- Implement RSS feed management (beyond basic auto-download rules).
- Provide a web-accessible remote interface (Tauri is a desktop context).

**UI patterns Tsubasa avoids:**
- Hamburger menus. Everything has a permanent place.
- Tabs within tabs. One level of tab navigation maximum.
- Infinite scroll. Tables are paginated or virtualized with explicit counts.
- Skeleton loaders. If data loads in under 200ms (which it should), show nothing. If longer, show a minimal spinner.
- Floating action buttons. Desktop interfaces don't need them.

**Visual trends Tsubasa rejects:**
- Glassmorphism with heavy blur. Light frosted-glass backdrop is acceptable for modals only.
- Neon/cyberpunk glow. Our glow is subtle — 1px spread, 20% opacity, accent color only.
- Rounded corners above 14px. Cards are 10-12px. Modals are 14px. Nothing is a pill except badges.
- Gradient text. Ever.
- Isometric illustrations. Onboarding uses typography and minimal shapes.

### Product Positioning

**Who it is for:**
People who download regularly, value speed and reliability, and are frustrated by the stagnation of traditional torrent clients. They may or may not understand BitTorrent mechanics — but they appreciate an interface that doesn't waste their time.

**Why it exists:**
Because qBittorrent looks like it was designed in 2008 (because it was). Because Transmission is beautiful but feature-poor. Because no torrent client has integrated cloud debrid providers as a first-class download path. Because desktop applications can be *good* now — Tauri proved that native + web can produce something fast, small, and well-designed.

**How it differs:**

| Dimension | qBittorrent | Transmission | Tsubasa |
|-----------|------------|-------------|---------|
| Cloud acceleration | ✗ | ✗ | ✓ Torbox, Real-Debrid — first-class |
| Smart routing | ✗ | ✗ | ✓ Automatic cloud/local/hybrid |
| Visual design | Legacy Qt | Clean but dated | Modern dark-first, three themes |
| Search | Built-in plugin | None | Torrent API integration |
| Data density | Moderate | Low | High, with progressive disclosure |
| Stack | C++ / Qt | C / GTK | Rust / React (Tauri) |

**How hybrid cloud defines it:**
Cloud debrid is not a plugin — it's the *thesis*. Tsubasa treats cloud providers as parallel download paths that compete with the local swarm. Smart Mode chooses the fastest path automatically. The user doesn't configure this — they just add a torrent and it works. The source badge on each torrent row tells them *how* it's being downloaded, but only if they care to look.

---

## Part 2 — Visual Identity System

### Color Architecture

#### Deep Black Theme (Default)

The base palette is built on absolute depth. Not `#1a1a1a` gray — true dark with enough warmth to avoid clinical coldness.

```css
/* ── Foundations ── */
--base:         hsl(240, 6%, 6%);       /* #0e0e10 — deepest layer */
--surface:      hsl(240, 5%, 9%);       /* #141417 — panels, sidebar */
--overlay:      hsl(240, 4%, 12%);      /* #1d1d21 — cards, inputs, dropdowns */
--muted:        hsl(240, 4%, 16%);      /* #26262b — hover states */
--subtle:       hsl(240, 3%, 20%);      /* #323236 — pressed states, borders */

/* ── Foreground ── */
--fg:           hsl(0, 0%, 93%);        /* #ededed — primary text */
--fg-2:         hsl(0, 0%, 60%);        /* #999999 — secondary text */
--fg-3:         hsl(0, 0%, 40%);        /* #666666 — tertiary, hints */

/* ── Borders ── */
--line:         hsla(0, 0%, 100%, 0.06);
--line-strong:  hsla(0, 0%, 100%, 0.10);
--line-subtle:  hsla(0, 0%, 100%, 0.03);

/* ── Accent ── */
--accent:       hsl(217, 92%, 62%);     /* #3b82f6 — interactive blue */
--accent-hover: hsl(217, 92%, 57%);     /* slightly darker on hover */
--accent-soft:  hsla(217, 92%, 62%, 0.12); /* focus rings, selected backgrounds */
--accent-glow:  hsla(217, 92%, 62%, 0.20); /* 1px box-shadow glow */

/* ── Semantic ── */
--green:        hsl(142, 71%, 45%);     /* success, seeding, complete */
--green-glow:   hsla(142, 71%, 45%, 0.25);
--amber:        hsl(38, 92%, 50%);      /* warning, attention, stalled */
--amber-glow:   hsla(38, 92%, 50%, 0.20);
--red:          hsl(0, 72%, 51%);       /* error, dead */
--red-glow:     hsla(0, 72%, 51%, 0.20);
```

#### Surface Elevation Logic

```
Layer 0: --base      → Window background, empty space
Layer 1: --surface   → Sidebar, main panels, settings body
Layer 2: --overlay   → Inputs, dropdown menus, cards within panels
Layer 3: --muted     → Hover states on Layer 2 items
Layer 4: --subtle    → Pressed states, active drags
```

**Rule:** Never skip a layer. If a card lives on `--base`, its background is `--surface`, and inputs within it use `--overlay`. If a modal floats above `--surface`, its background is `--overlay`, and inputs within it use `--muted`.

#### Glow Usage Rules

Glow is a **1px box-shadow** at 20% opacity of the relevant color. It signals "active" or "live."

- **Permitted on:** Active sidebar icon, live engine status dot, progress bar track, focused input border.
- **Forbidden on:** Static labels, disabled elements, decorative containers, text.
- **Maximum simultaneous glows:** 3 visible elements on screen. If more glow, reduce to the most important.

#### Rosé Pine Theme

```css
[data-theme="rose-pine"] {
  --base:         hsl(249, 22%, 12%);     /* #191724 */
  --surface:      hsl(247, 23%, 15%);     /* #1f1d2e */
  --overlay:      hsl(248, 25%, 18%);     /* #26233a */
  --muted:        hsl(249, 15%, 28%);     /* #3e3a56 */
  --fg:           hsl(245, 50%, 91%);     /* #e0def4 */
  --fg-2:         hsl(249, 12%, 47%);     /* #6e6a86 */
  --fg-3:         hsl(251, 10%, 38%);     /* #56526e */
  --accent:       hsl(340, 60%, 65%);     /* rose — #eb6f92 */
  --accent-soft:  hsla(340, 60%, 65%, 0.12);
  --green:        hsl(148, 30%, 60%);     /* pine */
  --amber:        hsl(35, 88%, 72%);      /* gold */
  --red:          hsl(343, 76%, 68%);     /* love */
}
```

#### Clean White Theme

```css
[data-theme="clean-white"] {
  --base:         hsl(0, 0%, 98%);        /* #fafafa */
  --surface:      hsl(0, 0%, 100%);       /* #ffffff */
  --overlay:      hsl(0, 0%, 96%);        /* #f5f5f5 */
  --muted:        hsl(0, 0%, 92%);        /* #ebebeb */
  --fg:           hsl(0, 0%, 9%);         /* #171717 */
  --fg-2:         hsl(0, 0%, 40%);        /* #666666 */
  --fg-3:         hsl(0, 0%, 60%);        /* #999999 */
  --line:         hsla(0, 0%, 0%, 0.08);
  --line-strong:  hsla(0, 0%, 0%, 0.14);
  --accent:       hsl(217, 92%, 50%);     /* slightly darker blue for contrast */
  --accent-soft:  hsla(217, 92%, 50%, 0.08);
}
```

#### Token Naming System

```
--{category}              → base value (--fg, --accent)
--{category}-{variant}    → intensity shift (--fg-2, --accent-hover)
--{category}-{modifier}   → purpose shift (--accent-soft, --green-glow)
--{structure}             → layout token (--line, --base, --surface)
--{structure}-{weight}    → intensity (--line-strong, --line-subtle)
```

### Typography System

#### Font Stack

```css
--font-sans:  'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
--font-mono:  'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
```

Inter is loaded at weights 400 (regular), 500 (medium), 600 (semibold). No bold (700) — semibold is the ceiling.

JetBrains Mono is loaded at weight 400 only.

#### Scale Hierarchy

| Token | Size | Weight | Line Height | Use |
|-------|------|--------|-------------|-----|
| `--text-xs` | 10px | 400 | 14px | Badges, status dots, footnotes |
| `--text-sm` | 11px | 400 | 16px | Table cells, secondary data, status bar |
| `--text-base` | 12px | 400 | 18px | Default body text, buttons, inputs, nav items |
| `--text-md` | 13px | 500 | 20px | Section titles, panel headers |
| `--text-lg` | 15px | 600 | 22px | Modal titles, onboarding headings |
| `--text-xl` | 18px | 600 | 26px | Welcome screen hero text only |

**Rules:**
- No font size between defined scale steps. If 12px is too small and 13px too large for a context, use 12px with medium weight.
- Data columns (speeds, sizes, peer counts) always use `--font-mono` at `--text-sm` with `font-variant-numeric: tabular-nums`.
- Table headers use `--text-xs`, weight 500, uppercase tracking `0.5px`, color `--fg-3`.

### Icon System

**Library:** Lucide React

| Context | Size | Stroke | Color |
|---------|------|--------|-------|
| Sidebar nav | 18px | 1.5 | `--fg-2`, active: `--accent` |
| Toolbar buttons | 15px | 1.5 | `--fg-2`, hover: `--fg` |
| Table row actions | 14px | 1.5 | `--fg-3`, hover: `--fg` |
| Badges | 12px | 1.5 | Inherits badge color |
| Settings section | 16px | 1.5 | `--fg-2` |

**Hover behavior:** Icon color transitions from `--fg-2` to `--fg` over 120ms ease-out. No scale transform.

**Active glow behavior:** Active sidebar icon gets a `0 0 6px var(--accent-glow)` box-shadow and color becomes `--accent`. The glow has a 200ms fade-in.

**Status badge system:**

| State | Background | Text | Border |
|-------|-----------|------|--------|
| Downloading | `--accent-soft` | `--accent` | none |
| Seeding | `hsla(142,71%,45%,0.12)` | `--green` | none |
| Paused | `--muted` | `--fg-2` | none |
| Queued | `--muted` | `--fg-3` | none |
| Error | `hsla(0,72%,51%,0.12)` | `--red` | none |
| Complete | `hsla(142,71%,45%,0.12)` | `--green` | none |
| Cloud | `hsla(280,60%,60%,0.12)` | `hsl(280,60%,65%)` | none |

### Spacing & Rhythm

#### Grid

Base unit: **4px**. All spacing values are multiples.

```
--space-1:  4px    (tight inner gaps)
--space-2:  8px    (standard inner gap, icon-label gap)
--space-3:  12px   (component padding, section gap)
--space-4:  16px   (panel padding, card padding)
--space-5:  20px   (modal padding, large section gap)
--space-6:  24px   (section separators, major gaps)
--space-8:  32px   (page-level margins)
```

#### Density Control

| Mode | Row height | Font size | Gap | Use case |
|------|-----------|-----------|-----|----------|
| **Compact** | 30px | 11px | 4px | Power users, many torrents |
| **Comfortable** | 38px | 12px | 8px | Default, balanced readability |

Toggle is a single icon button in the toolbar. Preference persists in settings DB.

---

## Part 3 — Motion & Interaction System

### Philosophy

Motion in Tsubasa serves exactly one purpose: **confirming that the system heard you.** It is never decorative. Never playful. It is the interface's way of saying *"acknowledged"* with the same composure as a flight status board updating a gate number.

Every animation answers one question: *"What just changed?"*

If removing the animation makes the transition confusing, it stays. If removing it changes nothing, it's deleted.

### Timing Scale

| Token | Duration | Use |
|-------|----------|-----|
| `--duration-instant` | 80ms | Color shifts on hover, opacity changes |
| `--duration-fast` | 150ms | Button press feedback, toggle switches |
| `--duration-normal` | 220ms | Panel slide, modal entrance, sidebar collapse |
| `--duration-slow` | 350ms | Theme transition, onboarding step change |

**Rules:**
- No animation exceeds 400ms. Desktop users penalize sluggishness more than mobile users.
- Entrance animations are 20% slower than exit animations. Things arrive with a moment of weight; they leave quickly.
- Animations that run during active data transfer (progress bars, speed counters) use 80ms or `transition: none` to avoid jank.

### Easing Curves

```css
--ease-out:       cubic-bezier(0.16, 1, 0.3, 1);      /* default exit, decelrate */
--ease-in-out:    cubic-bezier(0.65, 0, 0.35, 1);      /* symmetric transitions */
--ease-spring:    cubic-bezier(0.34, 1.56, 0.64, 1);   /* subtle overshoot for modals */
```

- `--ease-out` for most transitions (hover, color, slide).
- `--ease-spring` only for modal entrance and toast entrance — a 2-3px overshoot that settles.
- Never use linear easing.

### Specific Interactions

#### Sidebar Collapse

```
Expanded → Collapsed:
  Width: 200px → 52px over 220ms ease-out
  Labels: opacity 1 → 0 over 100ms (start immediately)
  Icons: translateX(0) — static, no motion
  Active indicator bar: width adjusts smoothly

Collapsed → Expanded:
  Width: 52px → 200px over 220ms ease-out
  Labels: opacity 0 → 1 over 120ms (delay 100ms, after width stabilizes)
```

Labels fade *before* the width shrinks, so text never clips or wraps. They fade *after* the width expands, so they appear into their final position.

#### Modal Entrance

```
Backdrop: opacity 0 → 0.6 over 200ms ease-out
Card:     opacity 0, scale(0.97), translateY(8px)
          → opacity 1, scale(1), translateY(0)
          over 250ms ease-spring

Exit:
Card:     opacity 1 → 0, scale(1 → 0.98) over 150ms ease-out
Backdrop: opacity 0.6 → 0 over 150ms ease-out (simultaneous)
```

The subtle scale + translate gives the modal a sense of emerging from depth without being dramatic.

#### Table Row Hover

```
Background: transparent → var(--muted) over 80ms ease-out
Row actions: opacity 0 → 1 over 80ms
```

No transform. No scale. No border change. Just a background shift that says "this row is focused."

#### Progress Bar

```
Track:  var(--muted), full width, height 3px, border-radius 2px
Fill:   var(--accent), width transitions with transition: width 300ms ease-out
Pulse:  At 100%, a single 0.6s opacity pulse (1 → 0.7 → 1) to indicate completion
```

No stripes. No gradients within the bar. No glow on the fill. The accent color is enough signal.

#### Toast Notifications

```
Entrance: translateX(40px), opacity 0 → translateX(0), opacity 1
          over 250ms ease-spring
Dwell:    5 seconds default
Exit:     translateX(40px), opacity 0 over 150ms ease-out
```

Stacking: new toasts push older ones up. Maximum 3 visible. Fourth toast replaces the oldest.

#### Theme Switching

```
All CSS variable transitions: 350ms ease-in-out
Applied via: transition: background-color, color, border-color, box-shadow 350ms
```

The entire interface crossfades between themes. No flash, no FOUC. The `data-theme` attribute change triggers all CSS variable transitions simultaneously.

---

## Part 4 — UI Reinvention

### Sidebar

**Structure:**

```
┌────────────────────┐
│  ≡  Tsubasa  翼    │  ← Brand mark, collapse toggle
├────────────────────┤
│  ↓  All             │  ← Category filters
│  ↓  Downloading     │     Icon + label
│  ↓  Seeding         │     Active: accent left bar
│  ↓  Completed       │     Counts are NOT shown
│  ∞  Cloud           │     (counts clutter the nav)
├────────────────────┤
│  🔍 Search          │  ← Search as navigation
├────────────────────┤
│                    │
│    (empty space)   │  ← Intentionally empty
│                    │     Sidebar breathes
├────────────────────┤
│  ⚙  Settings       │  ← Bottom-anchored
│  📊 Stats           │
└────────────────────┘
```

**Key decisions:**
- No torrent counts in sidebar. They change every second and create visual noise in a navigation context.
- Active state: 2px left border in `--accent`, background `--accent-soft`, icon/text color `--accent`.
- Collapsed mode: Icons only, 52px wide. Tooltip shows label on hover. Active indicator is full-height accent-colored left edge.
- The sidebar never shows contextual data. It is purely navigational.

### Dashboard — Torrent Table

**Column set (Comfortable mode):**

```
│ Name                          │ State    │ Progress │ Size    │ ↓ Speed  │ ↑ Speed  │ Source │ ETA    │
│ Inter 12px, --fg, truncate    │ Badge    │ Bar+%    │ Mono    │ Mono     │ Mono     │ Badge  │ Mono   │
```

**Source badges:**

| Source | Label | Color |
|--------|-------|-------|
| Local swarm | `Local` | `--fg-3` on `--muted` |
| Cloud (Torbox) | `Cloud` | purple tint |
| Hybrid (both) | `Hybrid` | accent tint |

**Progress bar in table:**
- 3px height, inline within the progress column.
- Track: `--muted`. Fill: `--accent` (downloading), `--green` (seeding/complete).
- Percentage text right-aligned, `--font-mono`, `--text-sm`.

**Empty state:**
When no torrents exist, the table area shows:
```
    Add your first torrent
    Paste a magnet link or drop a .torrent file

    [ Add Torrent ]
```

Centered, `--fg-3` text, single primary button. No illustration. No mascot.

### Detail Panel

Slides up from the bottom when a torrent row is selected. Height: 40% of viewport, resizable via drag handle.

**Tabs:** `Overview` · `Files` · `Peers` · `Trackers`

Tab transitions: content crossfades over 150ms. No slide animation (tabs are spatial, left-right slide implies order where there is none).

**Overview tab:**

```
┌─────────────────────────────────────────┐
│  Name: Ubuntu 24.04 LTS                 │  --fg, --text-md
│  Hash: a1b2c3d4...                      │  --fg-3, --font-mono, truncated
│                                         │
│  ┌─ Transfer ────────────────────┐      │
│  │  Downloaded   1.2 GB / 4.7 GB │      │
│  │  Uploaded     340 MB           │      │
│  │  Ratio        0.28             │      │
│  │  Time Active  2h 14m           │      │
│  └────────────────────────────────┘      │
│                                         │
│  ┌─ Network ─────────────────────┐      │
│  │  Peers   14 (42 available)    │      │
│  │  Seeds   8                     │      │
│  │  Source  Cloud (Torbox)        │      │
│  └────────────────────────────────┘      │
└─────────────────────────────────────────┘
```

Sections are grouped with subtle `--line-subtle` dividers and section headings in `--fg-3`, `--text-xs`, uppercase, 0.5px tracking.

### Add Torrent Modal

**Layout:**

```
┌──────────────────────────────────────┐
│  Add Torrent                     ✕   │
├──────────────────────────────────────┤
│                                      │
│  ┌──────────────────────────────┐    │
│  │  Paste magnet URI or URL     │    │  Tall text input, autofocus
│  └──────────────────────────────┘    │
│                                      │
│  ─── or ───                          │  Centered divider
│                                      │
│  ┌──────────────────────────────┐    │
│  │  Drop .torrent file here     │    │  Dashed border zone
│  └──────────────────────────────┘    │
│                                      │
│  ┌─ Download Mode ──────────────┐    │
│  │  ● Smart (recommended)      │    │  Radio cards
│  │  ○ Local Only               │    │  Each with one-line description
│  │  ○ Cloud Only               │    │
│  └──────────────────────────────┘    │
│                                      │
│  Save to: ~/Downloads    [Change]    │
│                                      │
│          [Cancel]    [Add Torrent]    │
└──────────────────────────────────────┘
```

**Behavior:**
- Pasting a magnet link auto-populates. If the hash is cached on a cloud provider, a subtle `⚡ Cached on Torbox` badge appears below the input within 500ms.
- Smart mode is preselected. The description reads: *"Automatically routes through the fastest available path."*
- Drop zone highlights with accent dashed border on drag-over.

### Settings

**Structure:** Left sidebar tabs, right content area (inside modal).

**Tab sections:**

```
General           → Download directory, launch at startup, notifications
Transfers         → Speed limits, max peers, ratio limits, queue rules
Cloud Providers   → Torbox API key, Real-Debrid API key, connection test
Smart Mode        → Cloud preference, fallback rules, cache priority
Appearance        → Theme selection (3 cards with preview), density toggle
Advanced          → Session persistence, cleanup rules, logging
```

**Theme selection:** Three cards side-by-side, each showing a miniature color swatch strip (base → surface → overlay → accent). Selected card has accent border. Clicking switches theme live with the 350ms crossfade.

**Cloud provider config:**
```
┌──────────────────────────────────────┐
│  Torbox                              │
│  API Key  [••••••••••]   [Test]      │
│  Status:  ● Connected                │
│  Plan:    Pro · Expires Mar 15       │
└──────────────────────────────────────┘
```

The `[Test]` button fires an `account_info` check. Green dot = connected. Red dot = failed. Shows plan info inline.

---

## Part 5 — Feature Reinvention

### Smart Mode (Default)

Smart Mode is the default download strategy. It is not a feature — it is the product's thesis.

**Logic:**
1. Check if torrent is cached on any configured cloud provider.
2. If cached → cloud download (instant start, full speed from CDN).
3. If not cached → start local swarm AND submit to cloud provider simultaneously.
4. Whichever path completes first wins. The other is cancelled.

**UI:** A small `Smart` badge on the torrent row source column. If the user switches to Local Only or Cloud Only manually, the badge changes accordingly.

### Command Palette (Ctrl+K)

A Raycast-style command palette for power users.

**Commands:**
- `Add torrent` → opens add modal
- `Pause all` / `Resume all`
- `Search torrents...` → filters table
- `Open settings`
- `Switch theme → Deep Black / Rosé Pine / Clean White`
- `Toggle density → Compact / Comfortable`
- `Copy magnet link` (when a torrent is selected)

**UI:** Centered overlay, `--overlay` background, `--line-strong` border, search input at top, results below. Appears with the modal entrance animation (scale+translate).

### Cloud Cache Badge

When adding a torrent, if any configured cloud provider has it cached, show:

```
⚡ Instant — cached on Torbox
```

Appears below the magnet input in the Add Torrent modal. Green-tinted text with a subtle flash animation (opacity 0 → 1 over 200ms).

### Speed Analytics Panel

Accessible from the Stats sidebar item. Shows:

```
┌─────────────────────────────────────┐
│  Last 24 Hours                      │
│  ↓ 12.4 GB downloaded               │
│  ↑ 3.1 GB uploaded                  │
│  Ratio: 0.25                        │
│                                     │
│  ┌─ Speed over time ────────────┐   │
│  │  (simple line chart, no lib)  │   │  Canvas-rendered, 120 data points
│  └───────────────────────────────┘   │
│                                     │
│  Active transfers: 3                 │
│  Peak speed: 42.1 MB/s              │
└─────────────────────────────────────┘
```

Chart is a simple canvas line graph — no chart library dependency. 1px line, accent color, filled area below at 5% opacity.

### Drag-and-Drop Magnet Zone

The entire app window is a drop target for `.torrent` files. On drag-over:

```
Full-window overlay:
  Background: var(--base) at 90% opacity
  Center text: "Drop to add torrent"
  Dashed accent border, 2px, border-radius 14px, inset 20px
```

Appears over 150ms. Drops open the Add Torrent modal with the file pre-loaded.

### Torrent Health Indicator

A five-segment arc icon (like a Wi-Fi signal) next to the peer count:

| Segment count | Meaning |
|---------------|---------|
| 5/5 | Excellent — many seeds, fast |
| 3-4/5 | Good — adequate peers |
| 1-2/5 | Poor — few seeds, slow |
| 0/5 | Dead — no connections |

Rendered as a single 14px SVG. Color follows semantic rules: green (5), fg-2 (3-4), amber (1-2), red (0).

### Auto-Cleanup Rules

In Settings → Advanced:

```
After completion:
  ● Do nothing
  ○ Remove from list after [7] days
  ○ Remove and delete files after [30] days
  ○ Remove when ratio reaches [2.0]
```

Runs on app launch and every 6 hours. Cleanup actions are logged. No surprise deletions — a toast notification confirms each removal.

---

## Part 6 — Onboarding Experience

### Flow

5 steps. Skippable at any point via "Skip setup" link (bottom-right, `--fg-3`, subtle).

**Step 1: Welcome**
```
    翼

    Welcome to Tsubasa

    A modern torrent client with
    cloud acceleration.

              [Get Started]
```

Background: very subtle gradient animation (dark → slightly-less-dark, oscillating over 8 seconds). Not a particle effect. Not a wave. Just a gentle depth shift like slow breathing.

**Step 2: Download Location**
```
    Where should files go?

    ~/Downloads/Tsubasa    [Change]

    You can always change this later in Settings.

    [Back]                        [Continue]
```

The default path is pre-filled with platform-appropriate directory.

**Step 3: Cloud Providers (Optional)**
```
    Accelerate with cloud providers

    Connect a debrid service for instant
    downloads of cached torrents.

    ┌─ Torbox ─────────────────────┐
    │  API Key  [____________]     │
    │                   [Connect]  │
    └──────────────────────────────┘

    ┌─ Real-Debrid ────────────────┐
    │  API Key  [____________]     │
    │                   [Connect]  │
    └──────────────────────────────┘

    Skip — you can add these anytime.

    [Back]                        [Continue]
```

**Step 4: Smart Mode**
```
    How Tsubasa downloads

    Smart Mode automatically picks
    the fastest download path.

       ┌──────┐    ┌──────┐
       │ Local│ or │ Cloud│ ← whichever
       │Swarm │    │ CDN  │   is faster
       └──────┘    └──────┘
              ↘    ↙
            Your disk

    [Back]                        [Continue]
```

Simple diagram. No library. Just styled divs with connecting lines.

**Step 5: Theme**
```
    Choose your look

    ┌────────┐  ┌────────┐  ┌────────┐
    │ Deep   │  │ Rosé   │  │ Clean  │
    │ Black  │  │ Pine   │  │ White  │
    │ ■■■■■  │  │ ■■■■■  │  │ ■■■■■  │
    └────────┘  └────────┘  └────────┘

    Selected theme applies instantly.

    [Back]                   [Finish Setup]
```

Each card shows the five-swatch strip (base → surface → overlay → accent → fg). Selected card has accent border + checkmark.

---

## Part 7 — Implementation System

### CSS Variable Token System

All tokens live in `index.css` under `[data-theme]` selectors. No Tailwind color utilities — all color references go through CSS variables.

```css
:root,
[data-theme="deep-black"] {
  /* Foundation tokens from Part 2 */
  --base: hsl(240, 6%, 6%);
  /* ... */

  /* Spacing tokens */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;

  /* Typography tokens */
  --text-xs: 10px;
  --text-sm: 11px;
  --text-base: 12px;
  --text-md: 13px;
  --text-lg: 15px;
  --text-xl: 18px;

  /* Timing tokens */
  --duration-instant: 80ms;
  --duration-fast: 150ms;
  --duration-normal: 220ms;
  --duration-slow: 350ms;

  /* Easing tokens */
  --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
  --ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);

  /* Radius tokens */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
  --radius-xl: 12px;
  --radius-2xl: 14px;
  --radius-full: 9999px;

  /* Layout tokens */
  --sidebar-width: 200px;
  --sidebar-collapsed: 52px;
  --toolbar-height: 44px;
  --statusbar-height: 28px;
  --detail-panel-min: 200px;
  --detail-panel-default: 40%;
}
```

### Component Abstraction

Each component follows this structure:

```
src/components/
  ComponentName/
    ComponentName.tsx       ← React component
    ComponentName.css       ← Component-scoped styles (using CSS variables)
```

**Rules:**
- No inline styles except for truly dynamic values (progress bar width, resize handle position).
- All color, spacing, and timing values reference tokens. Zero hardcoded values.
- Component CSS uses BEM-like scoping: `.torrent-table__row`, `.sidebar__nav-item`.
- Framer Motion only for enter/exit animations (mount/unmount). CSS transitions for hover/focus/active.

### Theme Switching Architecture

```
1. User selects theme in Settings or Onboarding
2. ThemeStore.setTheme("rose-pine") called
3. Store sets document.documentElement.setAttribute("data-theme", "rose-pine")
4. Store persists to Tauri backend: setSetting("theme", "rose-pine")
5. CSS variables transition over 350ms (handled by CSS `transition` on :root)
6. On next launch: loadSettings() → apply stored theme before first paint
```

Critical: Theme must be applied **before React hydration** to prevent flash of wrong theme. The Rust backend sends the theme in the initial webview configuration, and `index.html` has an inline script that applies it:

```html
<script>
  document.documentElement.setAttribute(
    'data-theme',
    localStorage.getItem('tsubasa-theme') || 'deep-black'
  );
</script>
```

### Performance Guardrails

1. **Virtual table.** The torrent table uses `@tanstack/react-virtual` for row virtualization above 50 rows. DOM never contains more than viewport + 10 buffer rows.

2. **Throttled progress updates.** The 500ms poll interval on the Rust side means React receives at most 2 updates/second per torrent. The store batches updates with `requestAnimationFrame`.

3. **Debounced search.** Command palette and table search debounce input by 150ms before filtering.

4. **No re-renders from theme.** Theme is pure CSS — changing `data-theme` triggers zero React re-renders.

5. **Lazy detail panel.** The Peers and Trackers tabs fetch data on-demand when activated, not on torrent selection.

### State Architecture

```
Zustand stores (frontend):
  torrentStore    → torrent list, selection, sort/filter state
  themeStore      → current theme, density mode
  settingsStore   → cached settings (loaded on mount)
  uiStore         → sidebar collapsed, detail panel height, modals open

Rust state (backend):
  AppState        → engine, DB, cloud manager, event bus, orchestrator

Communication:
  Frontend → Backend:  Tauri invoke (IPC commands)
  Backend → Frontend:  Event bus (broadcast::Sender → Tauri event emit)
```

**Anti-desync rules:**
- Torrent state is **always** read from the backend on `get_torrents`. The frontend store is a cache, not source of truth.
- Optimistic updates are used for `pause`/`resume` (instant UI feedback), but the store reconciles on the next `get_torrents` poll (every 2 seconds).
- Settings changes emit an event that all listening stores receive, preventing stale reads.

---

## Part 8 — Self-Critique

### Areas that risk feeling generic

1. **The sidebar structure** follows a well-worn pattern (nav items, bottom-anchored settings). Differentiation comes from what we *exclude* (no counters, no badges, no mini-graphs) rather than novel structure. This is intentional — navigation patterns work because they're learned, and novelty in navigation is a usability tax.

2. **The color palette** (dark base + blue accent) is the safest choice in desktop app design. Rosé Pine variant adds personality. But the default dark theme could feel like "another Vercel clone" if executed without the specific HSL values and glow rules documented here. **Mitigation:** The accent glow system and the 240° hue on the base (slightly blue-shifted black) create subtle warmth that distinguishes it from true-neutral darks.

3. **Modal layout** (centered card with backdrop) is universal. **Mitigation:** The entrance animation (scale 0.97 + 8px translate + spring easing) gives it a specific feel that differs from generic ease-in-out fades.

### Cliché patterns removed

- ❌ **Dashboard cards with big numbers.** "Total Downloaded: 142 GB" in a hero card. This is vanity — it doesn't help the user do anything. Replaced with inline stats in the detail panel and the stats view.
- ❌ **Animated counters.** Rolling number animations on speed values. These cause visual jitter on rapidly updating numbers. Use static renders with `tabular-nums`.
- ❌ **Sidebar collapse with hamburger icon.** The collapse toggle is a small arrow chevron at the top of the sidebar, not a hamburger menu.
- ❌ **"Built with ❤️" footer.** No footer. No attribution chrome. The status bar shows useful data (engine status, peer count, speeds).
- ❌ **Preview thumbnails.** Torrent clients that try to show media previews are overstepping their purpose. Tsubasa is a transfer instrument.

### Final refinements

- The onboarding welcome screen's "breathing" background gradient must be tested on low-end hardware. If it causes frame drops, replace with a static gradient.
- The command palette should feel native — same speed as Spotlight/Raycast. Input-to-filter latency must be under 16ms.
- The health indicator arc SVG should be pixel-hinted at 14px to avoid anti-aliasing blur at small sizes.
- Toast notification slide-in should use `will-change: transform` for GPU compositing, then remove the property after the animation completes.
- The cloud cache check in the Add Torrent modal must not block the UI. It fires asynchronously after the user pastes the magnet, and the badge appears when results arrive.

---

*This document is the source of truth for Tsubasa's identity, visual language, and implementation standards. Every design decision, code review, and feature proposal should be measured against these principles.*
