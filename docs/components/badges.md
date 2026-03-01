# Badges

## Overview

Badges are small pill-shaped status indicators used in table rows, the sidebar, and the detail panel. They communicate state visually through color.

---

## Base Spec

Defined as the `.badge` class in `globals.css`:

```css
.badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 6px;
  border-radius: var(--radius-full);  /* 9999px */
  font-size: 10px;
  font-weight: 500;
  line-height: 1.6;
}
```

---

## Variants

| Class          | Background          | Text Color       | Use Case                               |
|----------------|---------------------|------------------|----------------------------------------|
| `.badge-green` | `var(--green-soft)` | `var(--green)`   | Completed, seeding, connected, success |
| `.badge-red`   | `var(--red-soft)`   | `var(--red)`     | Errored, failed, danger                |
| `.badge-amber` | `var(--amber-soft)` | `var(--amber)`   | Queued, pending, checking, warning     |
| `.badge-accent`| `var(--accent-soft)`| `var(--accent)`  | Downloading, active, highlighted       |
| `.badge-blue`  | `var(--blue-soft)`  | `var(--blue)`    | Cloud, information                     |
| `.badge-muted` | `var(--muted)`      | `var(--fg-2)`    | Stopped, paused, inactive              |

---

## Usage Examples

### Table Status Column

```jsx
function getStatusBadge(state: TorrentState) {
  const map: Record<string, { className: string; label: string }> = {
    downloading: { className: "badge badge-accent", label: "Downloading" },
    seeding:     { className: "badge badge-green",  label: "Seeding" },
    completed:   { className: "badge badge-green",  label: "Completed" },
    paused:      { className: "badge badge-muted",  label: "Paused" },
    errored:     { className: "badge badge-red",    label: "Error" },
    queued:      { className: "badge badge-amber",  label: "Queued" },
    pending:     { className: "badge badge-amber",  label: "Pending" },
    checking:    { className: "badge badge-amber",  label: "Checking" },
    stopped:     { className: "badge badge-muted",  label: "Stopped" },
  };
  const entry = map[state] ?? { className: "badge badge-muted", label: state };
  return <span className={entry.className}>{entry.label}</span>;
}
```

### Cloud Provider Status

```jsx
<span className={configured ? "badge badge-green" : "badge badge-muted"}>
  {configured ? "Configured" : "Not configured"}
</span>
```

### Sidebar Count Badge

The sidebar count badges are not the `.badge` class. They use a custom inline style with `var(--muted)` background and monospace font. See `sidebar.md` for the full spec.

---

## State-to-Badge Mapping

| Torrent State | Badge Variant | Label       |
|---------------|---------------|-------------|
| `downloading` | `badge-accent`| Downloading |
| `pending`     | `badge-amber` | Pending     |
| `checking`    | `badge-amber` | Checking    |
| `queued`      | `badge-amber` | Queued      |
| `seeding`     | `badge-green` | Seeding     |
| `completed`   | `badge-green` | Completed   |
| `paused`      | `badge-muted` | Paused      |
| `stopped`     | `badge-muted` | Stopped     |
| `errored`     | `badge-red`   | Error       |

---

## Do and Don't

**Do:**
- Use badges only for categorical status. Never for numeric values.
- Keep badge labels to one or two words maximum.
- Include an icon inside the badge only when it adds meaning (e.g., a cloud icon for cloud status).
- Use the background+text color pairing from the table above. Never mix (e.g., green background with red text).

**Don't:**
- Use badges for inline counters. Use the sidebar count pill style instead.
- Create new badge variants. The 6 existing variants cover all needed states.
- Use badges for interactive elements. Badges are read-only indicators.
- Stack multiple badges in a single table cell.
