# Onboarding Flow

## Overview

The onboarding screen appears on first launch when the application has no configured settings. It guides the user through initial setup: choosing a download directory, configuring a cloud provider API key (optional), and selecting a theme.

---

## Layout

The onboarding replaces the normal dashboard. It is rendered as a full-screen modal over the base background.

```
+------------------------------------------------------------------+
|                                                                    |
|               +------- Card (480px) --------+                      |
|               |                              |                      |
|               |  [Logo]                      |                      |
|               |  Welcome to Tsubasa          |                      |
|               |                              |                      |
|               |  Step content...             |                      |
|               |                              |                      |
|               |  [Back]          [Continue]   |                      |
|               +------------------------------+                      |
|                                                                    |
+------------------------------------------------------------------+
```

---

## Card Spec

| Property      | Value                                    |
|---------------|------------------------------------------|
| Width         | 480px                                    |
| Background    | `var(--surface)`                         |
| Border        | `1px solid var(--line-strong)`           |
| Border radius | 16px (`--radius-2xl`)                    |
| Box shadow    | `var(--shadow-lg)`                       |
| Padding       | 32px                                     |
| Position      | Centered vertically and horizontally     |

---

## Steps

### Step 1: Welcome

- Logo badge (32x32, gradient, glow)
- Title: "Welcome to Tsubasa" (18px, weight 600)
- Subtitle: Description text (13px, `var(--fg-2)`)
- Single Continue button

### Step 2: Download Directory

- Section title: "Choose download folder" (14px, weight 500)
- Path display with a Browse button (invokes `dialog.open`)
- Shows current path in a read-only input field

### Step 3: Cloud Setup (Optional)

- Title: "Cloud Providers (Optional)" (14px, weight 500)
- Explanation text in `var(--fg-2)`
- Torbox API key input field
- Real-Debrid API key input field
- "Skip" button alongside "Continue"

### Step 4: Theme Selection

- Title: "Choose your theme"
- Three theme cards displayed vertically, each showing:
  - Theme name and description
  - Color swatch preview (3 circles: base, surface, accent)
  - Click to select, selected card gets accent ring
- Continue button labeled "Get Started"

---

## Navigation

| Button     | Style        | Position | Behavior                      |
|------------|-------------|----------|-------------------------------|
| Back       | Ghost        | Left     | Goes to previous step         |
| Continue   | Primary      | Right    | Validates and advances        |
| Skip       | Ghost        | Right    | Skips optional step           |
| Get Started| Primary      | Right    | Finalizes and closes onboarding|

Footer button area: flex, `justify-content: space-between`.

---

## Step Indicator

A row of small dots at the top of the card content:

```jsx
<div style={{ display: "flex", gap: 6, justifyContent: "center", marginBottom: 24 }}>
  {steps.map((_, i) => (
    <div
      key={i}
      style={{
        width: i === currentStep ? 18 : 6,
        height: 6,
        borderRadius: 99,
        background: i === currentStep ? "var(--accent)" : "var(--muted)",
        transition: "width 200ms cubic-bezier(0.4, 0, 0.2, 1), background 150ms ease",
      }}
    />
  ))}
</div>
```

The active dot stretches to 18px wide and uses the accent color. Inactive dots are 6px circles with muted background.

---

## Completion

When the user clicks "Get Started" on the final step:

1. Settings are saved to the backend via `updateSettings()`.
2. Theme is applied via `setTheme()`.
3. Onboarding flag is set so it does not show again.
4. The onboarding component unmounts and the normal dashboard renders.

---

## Conditional Display

The onboarding shows only when:
- No download directory is configured, OR
- The `onboarding_completed` flag is false in the backend settings.

Checked on app startup in `App.tsx`.

---

## Do and Don't

**Do:**
- Make cloud setup skippable. It is not required for basic torrenting.
- Show the current download path in the directory step so users know the default.
- Apply the selected theme in real time during step 4 so users see the preview immediately.

**Don't:**
- Show the onboarding over the dashboard. Replace the dashboard entirely.
- Block the app if the user closes the window during onboarding.
- Add more than 4 steps. Keep it fast.
- Require email, account creation, or network access during onboarding.
