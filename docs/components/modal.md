# Modal

## Overview

Modals are used for settings, configuration, and any action requiring focused attention. Tsubasa currently has one modal: the Settings panel. All future modals must follow the same spec.

---

## Structure

```
+------- Backdrop (fixed, inset 0) --------+
|                                           |
|     +--- Modal Card (centered) ----+      |
|     | Header (title + actions)     |      |
|     |------------------------------|      |
|     | Body (tabs + content)        |      |
|     +------------------------------+      |
|                                           |
+-------------------------------------------+
```

---

## Backdrop

| Property        | Value                               |
|-----------------|-------------------------------------|
| Position        | `fixed`, `inset: 0`                 |
| z-index         | 50                                  |
| Background      | `rgba(0, 0, 0, 0.65)`              |
| Backdrop filter  | `blur(6px)`                        |
| Click behavior  | Closes modal when clicking outside  |

Framer Motion animation:
```jsx
initial={{ opacity: 0 }}
animate={{ opacity: 1 }}
exit={{ opacity: 0 }}
transition={{ duration: 0.15 }}
```

---

## Card

| Property      | Value                                                           |
|---------------|-----------------------------------------------------------------|
| Width         | 660px                                                           |
| Max height    | 82vh                                                            |
| Background    | `var(--surface)`                                                |
| Border        | `1px solid var(--line-strong)`                                  |
| Border radius | 14px                                                            |
| Box shadow    | `var(--shadow-lg), 0 0 40px rgba(0,0,0,0.4)`                   |
| Overflow      | hidden                                                          |
| Display       | flex column                                                     |

Framer Motion animation:
```jsx
initial={{ opacity: 0, y: -16, scale: 0.97 }}
animate={{ opacity: 1, y: 0, scale: 1 }}
exit={{ opacity: 0, y: -16, scale: 0.97 }}
transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}
```

---

## Header

| Property      | Value                               |
|---------------|-------------------------------------|
| Padding       | 14px 20px                           |
| Border bottom | `1px solid var(--line)`             |
| Title size    | 14px, weight 600                    |
| Title color   | `var(--fg)`                         |
| Letter spacing| -0.2px                              |

The header contains: title text (left) and action buttons (right). Action buttons include a context-sensitive Save button (shown only when dirty) and a close X button.

### Close Button

- 30x30px, border-radius 7px
- Icon: `X` at 15px
- Hover: `background: var(--muted)`, `color: var(--fg-2)`

---

## Tab Sidebar (Settings-Specific)

| Property        | Value                           |
|-----------------|---------------------------------|
| Width           | 152px, fixed                    |
| Background      | `var(--base)`                   |
| Border right    | `1px solid var(--line)`         |
| Padding         | 8px 0                          |
| Item padding    | 8px 16px                        |
| Item font size  | 12px                            |
| Active bg       | `var(--accent-soft)`            |
| Active color    | `var(--accent)`                 |
| Active weight   | 500                             |
| Left accent bar | 3px wide, `var(--accent)`, glow |

---

## Content Area

| Property | Value        |
|----------|-------------|
| Flex     | 1           |
| Overflow | `auto` (y)  |
| Padding  | 20px 24px   |

---

## Keyboard Interaction

| Key    | Action                        |
|--------|-------------------------------|
| Escape | Closes the modal              |
| Tab    | Moves focus between form fields|

The `useEffect` hook registers a global `keydown` listener for Escape on mount and removes it on unmount.

---

## Do and Don't

**Do:**
- Wrap the modal in `<AnimatePresence>` for enter/exit transitions.
- Close on backdrop click and Escape key.
- Show a Save button only when the form is dirty.
- Use `position: fixed` with `inset: 0` for the backdrop.

**Don't:**
- Stack modals. Only one modal can be open at a time.
- Allow scrolling on the body behind the modal. The backdrop prevents interaction.
- Use a modal for confirmations that can be handled with a toast or inline feedback.
- Set the modal width above 700px. The 660px width fits comfortably on 1280px screens.
