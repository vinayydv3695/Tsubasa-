# Theme System

## Architecture

Tsubasa supports three themes: Deep Black, Rose Pine, and Clean White. The theme engine is CSS-variable based with runtime switching via a `data-theme` attribute on the `<html>` element.

---

## How It Works

### Layer 1: CSS Variable Definitions

Each theme is a block of CSS custom properties scoped to a `[data-theme]` selector in `globals.css`:

```css
[data-theme="black"], :root { /* Deep Black tokens */ }
[data-theme="rose"]         { /* Rose Pine tokens */ }
[data-theme="light"]        { /* Clean White tokens */ }
```

`:root` falls back to Deep Black, ensuring the app has a theme even if `data-theme` is unset.

### Layer 2: Tailwind Registration

Tokens are registered in a `@theme` block so Tailwind v4 generates utility classes:

```css
@theme {
  --color-base: var(--base);
  --color-surface: var(--surface);
  /* ... */
}
```

### Layer 3: Runtime Switching

The `useThemeStore` Zustand store manages the active theme:

```typescript
// src/stores/theme.ts
interface ThemeStore {
  theme: Theme;           // "black" | "rose" | "light"
  setTheme: (t: Theme) => void;
}
```

When `setTheme` is called:

1. `document.documentElement.setAttribute("data-theme", theme)` is called immediately.
2. The theme string is persisted to the Tauri backend via `setSetting("theme", theme)`.
3. All CSS variables update instantly because they are scoped to the `[data-theme]` selector.
4. The global `180ms ease` transition on `*` handles the visual crossfade.

### Layer 4: Persistence

On app startup, `useThemeStore` reads the stored theme from the backend:

```typescript
// Inside the store initializer
getSetting("theme").then((saved) => {
  if (saved) {
    document.documentElement.setAttribute("data-theme", saved);
    set({ theme: saved as Theme });
  }
});
```

If no theme is stored (first launch), Deep Black is used by default.

---

## Theme Switching Flow

```
User clicks theme card in Settings > Appearance
  -> setTheme("rose") called on Zustand store
    -> document.documentElement.setAttribute("data-theme", "rose")
    -> CSS variables update instantly
    -> Global 180ms transition animates all colors
    -> setSetting("theme", "rose") persists to backend
    -> Next app launch reads saved theme
```

Total latency: under 1ms for the visual switch. The backend persistence call is fire-and-forget.

---

## Adding a New Theme

To add a fourth theme:

1. Add a new `[data-theme="themename"]` block in `globals.css` with all required tokens (see `color-system.md` for the full list).

2. Add the theme value to the `Theme` type in `src/stores/theme.ts`:
   ```typescript
   export type Theme = "black" | "rose" | "light" | "themename";
   ```

3. Add a card entry in `SettingsPanel.tsx` inside the `AppearanceTab` component's `themes` array:
   ```typescript
   {
     value: "themename",
     label: "Theme Label",
     description: "Short description.",
     icon: <IconComponent size={15} />,
     preview: ["#base", "#surface", "#accent"],
   }
   ```

4. Test all components against the new theme. Check:
   - Shadow visibility (dark themes need heavier shadows)
   - Glow token opacity (adjust `--accent-glow` for the accent color's brightness)
   - Border visibility (`--line` opacity may need tuning)
   - Status colors (ensure `--green`, `--red`, `--amber` have sufficient contrast)

---

## CSS Variable Completeness Check

Every theme must define all of the following tokens. Missing any will cause visual breakage:

**Surfaces (5):** `--base`, `--surface`, `--overlay`, `--muted`, `--subtle`

**Foreground (4):** `--fg`, `--fg-2`, `--fg-3`, `--fg-muted`

**Accent (4):** `--accent`, `--accent-hover`, `--accent-soft`, `--accent-glow`

**Borders (3):** `--line`, `--line-subtle`, `--line-strong`

**Status Colors (10):** `--green`, `--green-soft`, `--green-glow`, `--amber`, `--amber-soft`, `--red`, `--red-soft`, `--red-glow`, `--blue`, `--blue-soft`

**Cloud (1):** `--cloud`

**Shadows (4):** `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-glow`

**Gradients (2):** `--gradient-surface`, `--gradient-accent`

**Total: 33 tokens per theme.**

---

## Do and Don't

**Do:**
- Always test theme changes against all three themes before merging.
- Use `data-theme` on `<html>`, not on `<body>` or inner elements.
- Keep the global `*` transition rule for smooth theme crossfades.
- Use the `no-theme-transition` class on elements animated by Framer Motion.

**Don't:**
- Nest `[data-theme]` selectors. Only one theme is active at a time, always on `<html>`.
- Store theme preference in `localStorage`. Use the Tauri backend settings API for persistence.
- Conditionally render different components per theme. All components must work with all themes via CSS variables alone.
- Add theme-specific CSS classes (e.g., `.dark-mode-header`). Components must be theme-agnostic.
