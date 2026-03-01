# Color System

All colors in Tsubasa are defined as CSS custom properties on `[data-theme]` selectors. The application ships three themes. Each theme defines the same set of variables so that components reference only token names, never raw values.

---

## Token Architecture

### Surface Tokens

Surface tokens form a 5-step grayscale ladder. Each step is slightly lighter (in dark themes) or slightly darker (in the light theme) than the previous.

```
--base      Deepest background. Never interactive.
--surface   Primary panel background (sidebar, modals, cards).
--overlay   Input fields, dropdown menus, elevated cards.
--muted     Hover backgrounds, subtle fills.
--subtle    Pressed/active backgrounds, strong fills.
```

### Foreground Tokens

```
--fg        Primary text. High contrast.
--fg-2      Secondary text. Labels, descriptions.
--fg-3      Tertiary text. Hints, placeholders, timestamps.
--fg-muted  Disabled text, decorative text.
```

### Accent Tokens

```
--accent        Primary interactive color. Buttons, active tabs, links.
--accent-hover  Hover variant. Slightly lighter/brighter.
--accent-soft   10-12% opacity background fill for selected states.
--accent-glow   20-25% opacity shadow for glow effects.
```

### Status Tokens

Each status color has a solid variant and a soft (background) variant:

```
--green      / --green-soft  / --green-glow    Success, seeding, connected
--amber      / --amber-soft                     Warning, queued, pending
--red        / --red-soft    / --red-glow       Error, danger, destructive
--blue       / --blue-soft                      Information, cloud
```

### Border Tokens

```
--line          Default borders (6-7% opacity)
--line-subtle   Light separators (3-4% opacity)
--line-strong   Emphasized borders (10-12% opacity)
```

### Shadow Tokens

```
--shadow-sm     Subtle elevation (inputs, small cards)
--shadow-md     Medium elevation (dropdowns, popovers)
--shadow-lg     High elevation (modals, dialogs)
--shadow-glow   Accent-colored glow (active buttons, focused inputs)
```

### Gradient Tokens

```
--gradient-surface   Subtle vertical surface gradient (panel headers)
--gradient-accent    Diagonal accent gradient (primary buttons, logo badge)
```

---

## Theme: Deep Black

The default theme. High contrast, true-black base, indigo accent.

```css
[data-theme="black"], :root {
  --base:            #09090b;
  --surface:         #111114;
  --overlay:         #18181c;
  --muted:           #222228;
  --subtle:          #2c2c34;

  --fg:              #ededf0;
  --fg-2:            #a0a0ab;
  --fg-3:            #65656f;
  --fg-muted:        #404048;

  --accent:          #6366f1;
  --accent-hover:    #818cf8;
  --accent-soft:     rgba(99, 102, 241, 0.10);
  --accent-glow:     rgba(99, 102, 241, 0.25);

  --line:            rgba(255, 255, 255, 0.06);
  --line-subtle:     rgba(255, 255, 255, 0.03);
  --line-strong:     rgba(255, 255, 255, 0.10);

  --green:           #22c55e;
  --green-soft:      rgba(34, 197, 94, 0.12);
  --green-glow:      rgba(34, 197, 94, 0.2);
  --amber:           #f59e0b;
  --amber-soft:      rgba(245, 158, 11, 0.12);
  --red:             #ef4444;
  --red-soft:        rgba(239, 68, 68, 0.12);
  --red-glow:        rgba(239, 68, 68, 0.2);
  --blue:            #3b82f6;
  --blue-soft:       rgba(59, 130, 246, 0.12);

  --shadow-sm:       0 1px 2px rgba(0,0,0,0.5), 0 0 1px rgba(0,0,0,0.3);
  --shadow-md:       0 4px 12px rgba(0,0,0,0.6), 0 1px 3px rgba(0,0,0,0.4);
  --shadow-lg:       0 8px 30px rgba(0,0,0,0.7), 0 2px 8px rgba(0,0,0,0.5);
  --shadow-glow:     0 0 20px rgba(99, 102, 241, 0.18);

  --gradient-surface: linear-gradient(180deg, #151518, #111114);
  --gradient-accent:  linear-gradient(135deg, #6366f1, #818cf8);
}
```

---

## Theme: Rose Pine

Warm, muted palette with lavender accent. Based on the Rose Pine color scheme.

```css
[data-theme="rose"] {
  --base:            #191724;
  --surface:         #1f1d2e;
  --overlay:         #26233a;
  --muted:           #2a2740;
  --subtle:          #312e48;

  --fg:              #e0def4;
  --fg-2:            #908caa;
  --fg-3:            #6e6a86;
  --fg-muted:        #524f67;

  --accent:          #c4a7e7;
  --accent-hover:    #d4bff7;
  --accent-soft:     rgba(196, 167, 231, 0.12);
  --accent-glow:     rgba(196, 167, 231, 0.25);

  --line:            rgba(224, 222, 244, 0.07);
  --line-subtle:     rgba(224, 222, 244, 0.04);
  --line-strong:     rgba(224, 222, 244, 0.12);

  --green:           #9ccfd8;
  --green-soft:      rgba(156, 207, 216, 0.12);
  --green-glow:      rgba(156, 207, 216, 0.2);
  --amber:           #f6c177;
  --amber-soft:      rgba(246, 193, 119, 0.12);
  --red:             #eb6f92;
  --red-soft:        rgba(235, 111, 146, 0.12);
  --red-glow:        rgba(235, 111, 146, 0.2);
  --blue:            #31748f;
  --blue-soft:       rgba(49, 116, 143, 0.12);

  --shadow-sm:       0 1px 2px rgba(0,0,0,0.4), 0 0 1px rgba(0,0,0,0.2);
  --shadow-md:       0 4px 12px rgba(0,0,0,0.5), 0 1px 3px rgba(0,0,0,0.3);
  --shadow-lg:       0 8px 30px rgba(0,0,0,0.6), 0 2px 8px rgba(0,0,0,0.4);
  --shadow-glow:     0 0 20px rgba(196, 167, 231, 0.18);

  --gradient-surface: linear-gradient(180deg, #221f30, #1f1d2e);
  --gradient-accent:  linear-gradient(135deg, #c4a7e7, #d4bff7);
}
```

---

## Theme: Clean White

Professional light theme. Deep foreground text on white surfaces, indigo accent.

```css
[data-theme="light"] {
  --base:            #f6f6f9;
  --surface:         #ffffff;
  --overlay:         #f0f0f5;
  --muted:           #e8e8ef;
  --subtle:          #d8d8e3;

  --fg:              #1a1a2e;
  --fg-2:            #4a4a66;
  --fg-3:            #7a7a99;
  --fg-muted:        #aeaec5;

  --accent:          #4f46e5;
  --accent-hover:    #6366f1;
  --accent-soft:     rgba(79, 70, 229, 0.08);
  --accent-glow:     rgba(79, 70, 229, 0.20);

  --line:            rgba(0, 0, 0, 0.07);
  --line-subtle:     rgba(0, 0, 0, 0.04);
  --line-strong:     rgba(0, 0, 0, 0.12);

  --green:           #16a34a;
  --green-soft:      rgba(22, 163, 74, 0.10);
  --green-glow:      rgba(22, 163, 74, 0.15);
  --amber:           #d97706;
  --amber-soft:      rgba(217, 119, 6, 0.10);
  --red:             #dc2626;
  --red-soft:        rgba(220, 38, 38, 0.10);
  --red-glow:        rgba(220, 38, 38, 0.15);
  --blue:            #2563eb;
  --blue-soft:       rgba(37, 99, 235, 0.10);

  --shadow-sm:       0 1px 3px rgba(0,0,0,0.08), 0 0 1px rgba(0,0,0,0.04);
  --shadow-md:       0 4px 16px rgba(0,0,0,0.10), 0 1px 4px rgba(0,0,0,0.06);
  --shadow-lg:       0 8px 32px rgba(0,0,0,0.14), 0 2px 8px rgba(0,0,0,0.08);
  --shadow-glow:     0 0 20px rgba(79, 70, 229, 0.12);

  --gradient-surface: linear-gradient(180deg, #ffffff, #f8f8fc);
  --gradient-accent:  linear-gradient(135deg, #4f46e5, #6366f1);
}
```

---

## CSS Variable Strategy

### Layer 1: Raw Variables

Defined per theme in `globals.css` using `[data-theme]` selectors. The `:root` fallback uses the Deep Black theme by default.

### Layer 2: Tailwind Registration

Variables are registered in a `@theme` block so Tailwind v4 can generate utility classes:

```css
@theme {
  --color-base: var(--base);
  --color-surface: var(--surface);
  --color-accent: var(--accent);
  /* ... */
}
```

This allows using `bg-base`, `text-fg-2`, `border-line` in Tailwind classes.

### Layer 3: Component Consumption

Components use variables in two ways:

1. **Inline styles** (preferred for dynamic values):
   ```jsx
   style={{ color: "var(--accent)" }}
   ```

2. **CSS utility classes** (for static patterns):
   ```jsx
   className="badge-green"   // uses --green-soft and --green
   className="btn-primary"   // uses --gradient-accent
   ```

### Rules

- Never write a raw hex color in a component. Always use `var(--token)`.
- Always test color changes against all three themes before merging.
- Glow tokens (`--accent-glow`, `--green-glow`, `--red-glow`) must only be used in `box-shadow`. Never use them as background colors.
- Soft tokens (`--accent-soft`, `--green-soft`) are for background fills at 8-12% opacity. They must be paired with their solid counterpart as the text color.
