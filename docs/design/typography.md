# Typography

## Font Stack

### Primary: Inter

Used for all UI text: labels, buttons, navigation, headings, body copy.

```css
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

Loaded from Google Fonts in `index.html` with weights 300, 400, 500, 600, 700.

### Monospace: JetBrains Mono

Used for all numeric data: file sizes, speeds, percentages, hashes, ports, timestamps.

```css
font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Cascadia Code', ui-monospace, monospace;
```

Loaded from Google Fonts with weights 400, 500.

---

## Type Scale

The scale is compact, designed for the information-dense desktop UI. All sizes are defined as CSS custom properties in the `@theme` block.

| Token        | Size  | Use Case                                               |
|--------------|-------|--------------------------------------------------------|
| `--text-2xs` | 10px  | Badges, meta labels, tracker status, tiny hints        |
| `--text-xs`  | 11px  | Sidebar items, field hints, secondary descriptions     |
| `--text-sm`  | 12px  | Table cells, button labels, input text, nav items      |
| `--text-base`| 13px  | Body text, default font size on `<body>`               |
| `--text-md`  | 14px  | Section headers, settings titles, settings modal header|
| `--text-lg`  | 16px  | Panel headings ("Settings", "About"), app name         |
| `--text-xl`  | 18px  | Onboarding step titles                                 |
| `--text-2xl` | 20px  | Empty-state headings                                   |
| `--text-3xl` | 24px  | Reserved for future use (landing pages, error screens)  |

---

## Weight Scale

| Weight | Value | Use Case                                                 |
|--------|-------|----------------------------------------------------------|
| Light  | 300   | Reserved. Not in active use.                             |
| Normal | 400   | Body text, secondary labels, inactive nav items          |
| Medium | 500   | Button labels, active nav items, table headers, badges   |
| Semi   | 600   | Section titles, headings, app name, modal titles         |
| Bold   | 700   | Section header labels (uppercase, e.g., "STATUS")        |

---

## Line Height

Base line height on `<body>` is `1.5`. The following overrides apply:

| Context           | Line Height | Reason                                         |
|-------------------|-------------|-------------------------------------------------|
| Body text         | 1.5         | Standard readable text                          |
| Table cells       | 1.4         | Denser rows without clipping                    |
| Badges            | 1.6         | Prevents text from touching the pill edges      |
| Headings          | 1.2         | Tighter for visual weight                       |
| Monospace data    | 1            | Tabular data aligned to fixed row heights       |

---

## Letter Spacing

| Context                      | Value     | Reason                                    |
|------------------------------|-----------|-------------------------------------------|
| Section header labels        | `0.8px`   | Uppercase small text needs room to breathe |
| App name ("Tsubasa")         | `-0.3px`  | Tighter tracking for brand weight         |
| Settings/modal headings      | `-0.2px`  | Slight tightening for authority feel      |
| All other text               | default   | Inter's native spacing is well-optimized  |

---

## Usage Rules

### Numeric Data

All numbers that represent measurements must use monospace:

```jsx
<span style={{
  fontFamily: "'JetBrains Mono', monospace",
  fontVariantNumeric: "tabular-nums"
}}>
  4.2 MB/s
</span>
```

The `fontVariantNumeric: "tabular-nums"` property ensures digits are fixed-width, preventing speed values from jiggling when numbers change.

Applies to:
- Download/upload speeds
- File sizes
- Percentages and progress values
- ETA values
- Port numbers
- Ratio values
- Peer/seed counts

### Font Smoothing

Antialiasing is enabled globally:

```css
body {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

---

## Do and Don't

**Do:**
- Use `--text-sm` (12px) as the default for all interactive elements.
- Use `font-weight: 500` for active/selected items to add visual weight without bolding.
- Apply `fontVariantNumeric: "tabular-nums"` to any column of numbers.
- Use Inter for all UI text, JetBrains Mono for data.

**Don't:**
- Use `--text-lg` or larger in table cells, sidebar items, or toolbar buttons.
- Mix monospace and proportional fonts within the same label.
- Use italic text anywhere in the UI. It reduces readability on small text.
- Set font-weight above 600 for body-level text.
