# Toast

## Overview

Toasts are non-blocking notifications that appear in the bottom-right corner of the viewport. They auto-dismiss after a timeout and can be manually dismissed.

---

## Visual Spec

| Property          | Value                                                     |
|-------------------|-----------------------------------------------------------|
| Position          | Fixed, bottom-right, 16px from edges                      |
| Width             | 340px                                                     |
| Background        | `var(--surface)`                                          |
| Border            | `1px solid var(--line)`                                   |
| Border left       | 3px solid (status color)                                  |
| Border radius     | 10px                                                      |
| Box shadow        | `var(--shadow-md)`                                        |
| Backdrop filter    | `blur(12px) saturate(160%)`                              |
| Padding           | 12px 14px                                                 |
| Gap (icon-to-text)| 10px                                                      |
| Max visible       | 3 toasts stacked with 8px gap                             |

---

## Left Accent Border

The 3px left border color matches the notification type:

| Type    | Border Color     | Icon              | Icon Pill Background  |
|---------|------------------|-------------------|-----------------------|
| success | `var(--green)`   | `CheckCircle`     | `var(--green-soft)`   |
| error   | `var(--red)`     | `AlertCircle`     | `var(--red-soft)`     |
| warning | `var(--amber)`   | `AlertTriangle`   | `var(--amber-soft)`   |
| info    | `var(--blue)`    | `Info`            | `var(--blue-soft)`    |

---

## Icon Pill

The icon sits inside a small colored pill:

```jsx
<div style={{
  width: 28, height: 28,
  borderRadius: 8,
  background: iconBg,  // e.g. var(--green-soft)
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: 0,
}}>
  <Icon size={14} color={iconColor} />
</div>
```

---

## Text Content

| Element     | Font Size | Color          | Weight |
|-------------|-----------|----------------|--------|
| Title       | 12px      | `var(--fg)`    | 500    |
| Description | 11px      | `var(--fg-3)`  | 400    |

Title and description are stacked vertically with `gap: 2px`.

---

## Close Button

| Property      | Value                        |
|---------------|------------------------------|
| Position      | Absolute, top-right          |
| Size          | 20x20px                      |
| Icon          | `X` at 12px                  |
| Color         | `var(--fg-3)`                |
| Hover color   | `var(--fg-2)`                |
| Hover bg      | `var(--muted)`               |
| Border radius | 4px                          |

---

## Animation

```jsx
<motion.div
  initial={{ opacity: 0, x: 60, scale: 0.95 }}
  animate={{ opacity: 1, x: 0, scale: 1 }}
  exit={{ opacity: 0, x: 60, scale: 0.95 }}
  transition={{ duration: 0.3, ease: [0.25, 0.46, 0.45, 0.94] }}
/>
```

Toasts slide in from the right, 300ms duration. This is the slowest transition in the system but justified by the non-blocking nature.

---

## Behavior

- Auto-dismiss after 4000ms (success/info) or 6000ms (error/warning).
- Manual dismiss via close button click.
- New toasts push older ones upward.
- Maximum 3 visible toasts. Older toasts are removed when the limit is exceeded.
- Toasts are rendered via `<AnimatePresence>` for proper exit animations.

---

## Store Integration

```typescript
// src/stores/toast.ts
interface Toast {
  id: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  description?: string;
}

interface ToastStore {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => void;
  removeToast: (id: string) => void;
}
```

Usage:
```typescript
useToastStore.getState().addToast({
  type: "success",
  title: "Torrent added",
  description: "Ubuntu 24.04 Desktop is now downloading.",
});
```

---

## Do and Don't

**Do:**
- Use toasts for confirmations of completed actions (added, saved, deleted).
- Use toasts for errors that the user should be aware of but do not block workflow.
- Show the torrent name or action subject in the description.

**Don't:**
- Use toasts for blocking errors that require user action. Use an inline error message instead.
- Show toasts for routine background events (progress updates, peer connections).
- Stack more than 3 toasts. If many events happen at once, batch them.
- Use toasts for questions or confirmations. Toasts are informational only.
