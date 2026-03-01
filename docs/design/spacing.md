# Spacing System

## Base Unit

The spacing system uses a 4px base unit. All spacing values are multiples of 4.

---

## Spacing Scale

| Token  | Value | Common Use                                           |
|--------|-------|------------------------------------------------------|
| `1`    | 4px   | Icon-to-text gap in badges, tight internal padding   |
| `1.5`  | 6px   | Button padding (vertical), gap between icon and label|
| `2`    | 8px   | Sidebar section padding, small gaps, input padding   |
| `2.5`  | 10px  | Nav item horizontal padding, ghost button padding    |
| `3`    | 12px  | Button horizontal padding, table cell padding, input horizontal padding |
| `3.5`  | 14px  | Card internal padding, settings section padding      |
| `4`    | 16px  | Section margin, settings content padding             |
| `5`    | 20px  | Modal content padding, settings area padding         |
| `6`    | 24px  | Section vertical gap in settings panels              |
| `8`    | 32px  | Major section separations                            |

---

## Border Radius Scale

Defined as `@theme` tokens in `globals.css`:

| Token           | Value  | Use Case                                           |
|-----------------|--------|-----------------------------------------------------|
| `--radius-sm`   | 4px    | Focus rings, small interactive elements              |
| `--radius-md`   | 6px    | Buttons, nav items, sidebar items, table context menu|
| `--radius-lg`   | 8px    | Input fields, progress bars, cloud provider cards    |
| `--radius-xl`   | 12px   | Modals, settings panel, large card containers        |
| `--radius-2xl`  | 16px   | Unused. Reserved for onboarding or splash screens.   |
| `--radius-full` | 9999px | Badges, pills, toggle switches, scrollbar thumbs    |

### Radius Rules

1. Floating containers (modals, popovers) use `--radius-xl` (12-14px).
2. Interactive elements (buttons, inputs, nav items) use `--radius-md` or `--radius-lg`.
3. Status indicators and pills use `--radius-full`.
4. Table rows and tab lists have no border radius (0px).

---

## Panel Dimensions

### Sidebar

| State     | Width | Transition                             |
|-----------|-------|----------------------------------------|
| Expanded  | 208px | `width 200ms cubic-bezier(0.4,0,0.2,1)` |
| Collapsed | 48px  | Same easing                             |

### Settings Modal

| Property    | Value                             |
|-------------|-----------------------------------|
| Width       | 660px                             |
| Max height  | 82vh                              |
| Tab sidebar | 152px fixed width                 |

### Detail Panel

Height is determined by the parent flex layout. The panel sits below the torrent table at a user-resizable split point. Minimum usable height: 200px.

### Toolbar

| Property | Value |
|----------|-------|
| Height   | 44px  |
| Padding  | 0 12px|

### Status Bar

| Property | Value |
|----------|-------|
| Height   | 28px  |
| Padding  | 0 12px|

---

## Grid and Layout Spacing

### Settings Panel Internal Grid

The settings panels use specific grid layouts:

```
General > Queue Limits:       grid-template-columns: 1fr 1fr 1fr;  gap: 12px
General > Policy buttons:     flex, gap: 8px, each flex: 1
Bandwidth > Speed limits:     flex-direction: column, gap: 14px
Cloud > Provider cards:       flex-direction: column, gap: 16px
Appearance > Theme cards:     flex-direction: column, gap: 8px
```

### Torrent Table

- Header height: 34px
- Row height: 34px
- Cell horizontal padding: 12px
- Progress bar height: 3px
- Badge pill height: 18px

---

## Do and Don't

**Do:**
- Use even multiples of 4 for all spacing.
- Keep internal padding consistent per component type (all buttons = 6px 12px).
- Use `gap` in flex/grid layouts instead of margin on children.

**Don't:**
- Use spacing values like 5px, 7px, 9px, 11px, 15px. They break the 4px grid.
  (Exception: 7px is used in some existing components for vertical input padding. Treat as a known deviation.)
- Add margin-bottom on the last child of a flex container. Use `gap` on the parent.
- Use padding on table cells that differs from the 12px standard.
