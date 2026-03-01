# Motion System

## Core Principle

Motion in Tsubasa is functional, not decorative. Every animation communicates a state change, provides spatial orientation, or confirms user input. If an animation does not serve one of these purposes, remove it.

---

## Timing Scale

| Name      | Duration | Use Case                                                  |
|-----------|----------|-----------------------------------------------------------|
| `instant` | 0ms      | Theme color crossfade on theme switch (handled by CSS)    |
| `snap`    | 100ms    | Nav item hover, sidebar item highlight                    |
| `fast`    | 120ms    | Button hover, icon color change, badge appear             |
| `normal`  | 150ms    | Input focus ring, toggle switch, border-color transitions |
| `smooth`  | 180ms    | Theme transition on all elements (background, border, shadow) |
| `ease`    | 200ms    | Sidebar width collapse/expand, tab underline slide        |
| `modal`   | 200ms    | Modal open/close (scale + opacity + translate)            |
| `toast`   | 300ms    | Toast slide-in/out animation                              |
| `pulse`   | 2000ms   | Status dot pulse ring (infinite loop)                     |

**Rule:** No UI transition should exceed 300ms. Desktop users expect immediate response. The only exception is the infinite `pulse` animation on status dots.

---

## Easing Curves

| Name           | Value                              | Use Case                       |
|----------------|------------------------------------|--------------------------------|
| `ease-default` | `ease`                             | Color/opacity transitions      |
| `ease-out`     | `ease-out`                         | Pulse ring expansion           |
| `ease-smooth`  | `cubic-bezier(0.4, 0, 0.2, 1)`    | Sidebar collapse, width change |
| `ease-modal`   | `cubic-bezier(0.25, 0.46, 0.45, 0.94)` | Modal entrance/exit       |
| `linear`       | `linear`                           | Spinner rotation               |

### Choosing an Easing Curve

- **Entering the screen:** Use `ease-modal`. Elements should decelerate as they arrive.
- **Leaving the screen:** Use `ease-modal` (same curve, reversed via Framer Motion exit).
- **Changing size/position:** Use `ease-smooth`. Sidebar collapse is the primary example.
- **Changing color/opacity:** Use `ease` (CSS default). Fast and natural.

---

## CSS Transition Classes

Defined in `globals.css`:

### `transition-colors-fast`

Applied to interactive elements that respond to hover. Overrides the global 180ms theme transition with 120ms for snappier feedback.

```css
.transition-colors-fast {
  transition: color 120ms ease,
              background-color 120ms ease,
              border-color 120ms ease,
              box-shadow 120ms ease,
              opacity 120ms ease !important;
}
```

**Use on:** buttons, nav items, close icons, tab buttons.

### `transition-theme`

Explicit 200ms transition for elements that should animate during theme switching but not during normal interaction.

```css
.transition-theme {
  transition: color 200ms ease,
              background-color 200ms ease,
              border-color 200ms ease,
              box-shadow 200ms ease,
              opacity 200ms ease !important;
}
```

### Global Theme Transition

All elements have a baseline 180ms transition on background-color, border-color, color, and box-shadow. This ensures smooth theme switching without per-element setup.

```css
*, *::before, *::after {
  transition:
    background-color 180ms ease,
    border-color 180ms ease,
    color 100ms ease,
    box-shadow 180ms ease;
}
```

### `no-theme-transition`

Opt-out class for elements where the global transition would interfere (e.g., Framer Motion animated elements, progress bars).

```css
.no-theme-transition,
.no-theme-transition * {
  transition: none !important;
}
```

---

## Framer Motion Patterns

### Modal Entrance

Used by `SettingsPanel` and any future dialog:

```jsx
// Backdrop
<motion.div
  initial={{ opacity: 0 }}
  animate={{ opacity: 1 }}
  exit={{ opacity: 0 }}
  transition={{ duration: 0.15 }}
/>

// Content card
<motion.div
  initial={{ opacity: 0, y: -16, scale: 0.97 }}
  animate={{ opacity: 1, y: 0, scale: 1 }}
  exit={{ opacity: 0, y: -16, scale: 0.97 }}
  transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}
/>
```

Design rationale: The card rises 16px and scales from 97% to 100%. This creates a "popping forward" feel without being distracting. The backdrop fades faster (150ms) than the card (200ms) so content appears to float up from a dim background.

### Toast Notification

```jsx
<motion.div
  initial={{ opacity: 0, x: 60, scale: 0.95 }}
  animate={{ opacity: 1, x: 0, scale: 1 }}
  exit={{ opacity: 0, x: 60, scale: 0.95 }}
  transition={{ duration: 0.3, ease: [0.25, 0.46, 0.45, 0.94] }}
/>
```

Toasts slide in from the right edge. The 300ms duration is the longest in the system, justified because toasts are non-blocking and the slide provides directional context (the toast came from the system notification tray area).

### Spinner

Used for loading states in settings and detail panels:

```jsx
<Loader2 size={20} style={{ animation: "spin 1s linear infinite" }} />
```

---

## Keyframe Animations

### `pulse-ring`

Infinite pulse for the engine-ready status dot in the status bar:

```css
@keyframes pulse-ring {
  0%   { transform: scale(1);   opacity: 0.4; }
  70%  { transform: scale(2.2); opacity: 0;   }
  100% { transform: scale(2.2); opacity: 0;   }
}
```

Applied via `.pulse-green::after`. The pseudo-element expands from 1x to 2.2x scale while fading out, creating a radar-ping effect. The 0% to 70% active range with a 30% hold at zero prevents visual flicker between cycles.

### `spin`

Standard 360-degree rotation for loading spinners:

```css
@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}
```

Duration: 1s, timing: linear, iteration: infinite.

---

## Interaction Patterns

### Button Press

```
hover  -> opacity: 0.92, box-shadow: glow variant
active -> translateY(1px), opacity: 0.85
```

The 1px downward shift simulates a physical button press. Combined with opacity reduction, it gives unmistakable "I pressed this" feedback.

### Input Focus

```
idle   -> border: 1px solid var(--line)
focus  -> border: 1px solid var(--accent),
          box-shadow: 0 0 0 3px var(--accent-soft)
```

The 3px focus ring appears outside the border, using the soft accent color. This makes focused inputs visible without changing layout.

### Toggle Switch

```
off    -> background: var(--muted), knob at left: 2px
on     -> background: var(--accent), knob at left: 18px,
          box-shadow: 0 0 8px var(--accent-glow)
```

Transition: `left 150ms cubic-bezier(0.4, 0, 0.2, 1)`. The glow on the "on" state provides additional confirmation beyond color alone.

### Sidebar Collapse

```
expanded -> width: 208px
collapsed -> width: 48px
transition: width 200ms cubic-bezier(0.4, 0, 0.2, 1)
```

Labels and badges fade out immediately (opacity is not transitioned). Only the width animates. This prevents text from wrapping awkwardly during the transition.

---

## Do and Don't

**Do:**
- Use `transition-colors-fast` on all hoverable elements.
- Wrap modals in `<AnimatePresence>` for proper exit animation.
- Use `ease-smooth` for any width/height transition.
- Keep infinite animations subtle (low opacity, slow duration).

**Don't:**
- Animate `width` or `height` with CSS transitions on frequently updating elements (use `transform: scale` instead).
- Use `transition: all` anywhere. Explicitly list the properties being transitioned.
- Add entrance animations to table rows. Tables populate instantly.
- Use spring physics. Desktop apps feel better with bezier curves than bouncy springs.
