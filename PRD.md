# PRD: Claude Usage Desktop Widget

## Overview

A lightweight, always-visible macOS desktop widget that displays Claude usage limits using pixel-art health bar indicators. Built with Tauri v2, it reads your Claude Code OAuth token from the macOS Keychain and fetches live usage data from Anthropic's usage API — no login, no setup, no browser extensions. Just launch it and see your 5-hour session, 7-day all-models, and Opus-specific usage at a glance, styled after retro video game health bars.

---

## Problem Statement

Claude Pro/Team users have usage limits across multiple dimensions (session, weekly all-models, weekly Sonnet-only), but the only way to check usage is by navigating to **Settings → Usage** in the Claude web UI. This creates friction — users either over-conserve tokens out of uncertainty, or hit limits unexpectedly mid-task. A persistent desktop indicator solves this by making usage always visible.

---

## Goals

1. **At-a-glance awareness** — Users should understand their usage state in under 1 second.
2. **Delightful aesthetic** — Pixel-art health bars make a utilitarian metric feel fun and personal.
3. **Non-intrusive** — The widget should be small, always-on-top optional, and never block workflow.
4. **Accurate and timely** — Data should refresh automatically and reflect the same values shown in the Claude UI.

## Non-Goals

- Managing or changing usage limits from the widget.
- Displaying conversation history or content.
- **Implementing a login flow.** The widget never collects email/password. It reuses Claude Code's existing OAuth token from the macOS Keychain.
- Supporting non-macOS platforms in v1 (though Tauri's cross-platform support makes this feasible for v2).

---

## Target User

Claude Pro or Team plan subscribers who use Claude heavily throughout the day and want passive visibility into their remaining usage budget.

---

## Data Model

The widget maps the API response fields to visual meters. The API returns the following structure from `GET https://api.anthropic.com/api/oauth/usage`:

### API Response → Widget Mapping

| API Field | Widget Label | Description |
|---|---|---|
| `five_hour` | **Current Session** | Rolling 5-hour usage window |
| `seven_day` | **Weekly (All Models)** | 7-day usage across all models |
| `seven_day_opus` | **Weekly (Opus)** | 7-day usage for Opus models specifically |

Each field contains:

| Property | Type | Description |
|---|---|---|
| `utilization` | number (0–100) | Usage percentage |
| `resets_at` | string (ISO 8601) or null | UTC timestamp when the limit resets |

Additional fields in the response (`seven_day_oauth_apps`, `iguana_necktie`) may be null and can be ignored, or dynamically shown if non-null in future API versions.

### Display Normalization

The widget converts `resets_at` ISO timestamps into human-readable relative strings:
- If reset is within 24 hours → "Resets in X hr Y min"
- If reset is more than 24 hours away → "Resets [Day] [Time]" (e.g., "Resets Fri 7:00 AM")
- If `resets_at` is null → "No active limit"

The widget dynamically shows/hides meters based on which API fields are non-null. For example, if the user's plan does not include Opus limits, `seven_day_opus` will be null and that meter is hidden.

---

## Authentication & Data Source

### The API Endpoint

The usage data is available via a JSON API that Claude Code itself uses internally:

```
GET https://api.anthropic.com/api/oauth/usage
```

**Headers:**
```
Accept: application/json
Content-Type: application/json
Authorization: Bearer sk-ant-oat01-...
anthropic-beta: oauth-2025-04-20
```

**Response:**
```json
{
  "five_hour": {
    "utilization": 14.0,
    "resets_at": "2026-03-31T20:59:59.943648+00:00"
  },
  "seven_day": {
    "utilization": 15.0,
    "resets_at": "2026-04-04T11:00:00.943679+00:00"
  },
  "seven_day_oauth_apps": null,
  "seven_day_opus": {
    "utilization": 8.0,
    "resets_at": null
  },
  "iguana_necktie": null
}
```

This eliminates the need for HTML scraping entirely. The response gives us exactly the data the widget needs: utilization percentages and ISO 8601 reset timestamps.

### Authentication: Piggyback on Claude Code

**Prerequisite:** The user must have Claude Code installed and logged in with their Claude subscription (Pro, Max, Team, or Enterprise). This is the only requirement.

**How it works:**
1. Claude Code stores its OAuth credentials in the **macOS Keychain** under the service name `Claude Code-credentials`.
2. The credential entry contains a JSON blob with an `accessToken` (format: `sk-ant-oat01-...`), a `refreshToken`, an `expiresAt` timestamp, and the `subscriptionType`.
3. On each fetch cycle, the widget reads the access token from the keychain and uses it as a `Bearer` token in the `Authorization` header.
4. If the token is expired, the widget uses the `refreshToken` to obtain a new access token via Anthropic's OAuth refresh endpoint, then writes the updated credentials back to the keychain — the same flow Claude Code itself performs.
5. If refresh fails (e.g., user logged out of Claude Code), the widget shows a status indicator prompting the user to run `claude login` in their terminal.

**Why this works well:**
- **Zero setup for the user.** If Claude Code is logged in, the widget works immediately.
- **No credentials to manage.** The widget never asks for email, password, or API keys. It reuses what's already securely stored.
- **Token refresh is automatic.** The OAuth refresh flow keeps the token alive indefinitely without user intervention.
- **Secure by default.** macOS Keychain is encrypted and requires user authorization for first access.

**Fallback: Manual Token Entry**

For users who don't have Claude Code installed, the widget offers a manual setup path:
1. Run `claude setup-token` in any terminal to generate a long-lived OAuth token.
2. Paste the token into the widget's settings screen.
3. The widget stores it in the macOS Keychain under its own service name (`claude-usage-widget`).

### Data Fetching

1. Read the OAuth access token from the macOS Keychain (`Claude Code-credentials`).
2. Make an authenticated `GET` request to `https://api.anthropic.com/api/oauth/usage`.
3. Parse the JSON response and normalize into the internal data model.
4. Cache the last successful response locally (JSON file) so the widget can display stale-but-recent data if a fetch fails.
5. **Refresh interval:** Every 5 minutes (configurable, minimum 1 minute).
6. On HTTP 401 → attempt token refresh via OAuth. On repeated failure → show "Re-login needed" indicator.

---

## Visual Design

### Layout

The widget renders as a compact vertical stack of three health bars:

```
┌──────────────────────────────────────┐
│  SESSION (5hr)                       │
│  ♥  ██░░░░░░░░░░░░  14%  used       │
│     Resets in 3 hr 42 min            │
│                                      │
│  ALL MODELS (7-day)                  │
│  ♥  ███░░░░░░░░░░░  15%  used       │
│     Resets Fri 7:00 AM               │
│                                      │
│  OPUS (7-day)                        │
│  ♥  █░░░░░░░░░░░░░   8%  used       │
│     Resets Mon 3:00 PM               │
└──────────────────────────────────────┘
```

### Pixel Art Health Bar Specification

Each meter consists of three elements rendered in pixel-art style:

#### 1. Heart Icon (left side)
- Pixel-art heart, approximately 16×16 pixels at 1x scale.
- **Color states:**
  - 0–25% used → Full red heart (healthy)
  - 26–50% used → Full red heart
  - 51–75% used → Red heart (slightly desaturated or with a warning tint)
  - 76–90% used → Orange/amber heart
  - 91–100% used → Grey/empty heart outline (critical)

#### 2. Health Bar (center)
- Pixel-art rounded-rectangle bar with a 2px black pixel border.
- Interior fill uses a solid color, left-aligned, representing **remaining** usage (100% − used%).
- **Fill color mapping:**
  - 0–50% used → Green (`#4CAF50` or pixel-art equivalent)
  - 51–75% used → Yellow (`#FFC107`)
  - 76–90% used → Orange (`#FF9800`)
  - 91–100% used → Red (`#F44336`)
- Empty portion of the bar is white/light grey.
- The bar should be rendered at a crisp pixel scale (no anti-aliasing, integer scaling only) to preserve the retro aesthetic.

#### 3. Text (right side of heart / below bar)
- **Percentage:** Displayed beside the heart, e.g., `14% used`. Use a pixel/monospace font (e.g., "Press Start 2P", "Silkscreen", or system monospace at small size).
- **Reset time:** Displayed below the bar in a smaller, muted pixel font. e.g., `Resets in 3 hr 42 min`.

### Color & Theme

| Element | Value |
|---|---|
| Background | Transparent or configurable (dark/light) |
| Bar border | `#000000` (2px pixel border) |
| Bar fill (healthy) | `#E74C3C` (red, matching the reference pixel art) |
| Bar empty | `#FFFFFF` with slight grey tint |
| Text primary | `#FFFFFF` (dark mode) / `#1A1A1A` (light mode) |
| Text secondary (reset) | `#888888` |

### Widget Dimensions

- **Default size:** ~300px wide × ~180px tall (for 3 meters).
- **Scalable:** Support 1x, 1.5x, 2x scaling via a config option to accommodate different display densities and user preferences.

---

## Interaction & Behavior

### Window Behavior
- **Always on top:** Optional toggle (default: off). The widget can float above other windows.
- **Draggable:** The widget can be repositioned anywhere on the desktop.
- **Position memory:** Remembers its last position between launches.
- **Click-through:** When not being interacted with, mouse events optionally pass through to windows beneath.

### Interactions
- **Hover on a meter:** Show a tooltip with the exact usage fraction if available (e.g., "14.2% of session limit used").
- **Click on widget:** Opens the Claude Settings → Usage page in the default browser.
- **Right-click / context menu:**
  - Refresh now
  - Settings (open config)
  - Toggle always-on-top
  - Quit

### Notifications (Optional, v1.1)
- System notification when any meter exceeds 80%.
- System notification when any meter exceeds 95%.
- Configurable thresholds.

---

## Technical Architecture

### Recommended Stack

| Layer | Technology | Rationale |
|---|---|---|
| Desktop framework | **Tauri v2** | Small binary (~5–10MB vs Electron's ~150MB). Ideal for an always-on widget. |
| Frontend | **HTML Canvas + TypeScript** | Pixel art rendered on canvas at integer scale for crispness. No framework needed for this scope. |
| Backend | **Rust** (Tauri backend) | Periodic HTTP requests to the usage API. First-class macOS Keychain access via `security-framework` crate. |
| Storage | **JSON file** (local) | Persist config, window position, cached last-known usage data. `~/.claude-usage-widget/config.json` |
| Auth | **macOS Keychain** (read Claude Code's stored OAuth token) | Zero-setup. Reuses existing credentials. Token refresh handled automatically. |

### Why Not Electron or Swift?

- **Electron:** Running a full Chromium instance 24/7 for three health bars is ~150MB of RAM and a ~150MB binary. Tauri uses the system webview and ships at a fraction of the size.
- **Swift + AppKit:** Would produce the smallest, most native result, but limits contributions to macOS/Swift developers. Tauri's web frontend is more accessible and still produces a very lightweight app.

### Process Architecture

```
┌─────────────────────┐
│   Tauri Webview      │  ← HTML Canvas renders pixel-art bars
│   (Frontend)         │  ← TypeScript handles animation + layout
└──────────┬──────────┘
           │ Tauri IPC (invoke commands)
┌──────────▼──────────┐
│   Rust Backend       │  ← Reads OAuth token from macOS Keychain
│                      │  ← Fetches GET /api/oauth/usage every 5 min
│                      │  ← Handles token refresh on 401
│                      │  ← Caches last response to disk
└──────────┬──────────┘
           │ HTTPS (Bearer token auth)
┌──────────▼──────────┐
│   api.anthropic.com  │
│   /api/oauth/usage   │  ← Returns JSON: utilization + resets_at
└─────────────────────┘
```

---

## Configuration

Stored in a local JSON config file (e.g., `~/.claude-usage-widget/config.json`):

```json
{
  "refresh_interval_seconds": 300,
  "always_on_top": false,
  "scale": 1.5,
  "theme": "dark",
  "position": { "x": 100, "y": 100 },
  "notifications": {
    "enabled": true,
    "thresholds": [80, 95]
  }
}
```

> **Note:** No auth configuration is stored here. The widget reads OAuth tokens directly from the macOS Keychain (`Claude Code-credentials`). If the user doesn't have Claude Code installed, a one-time manual token entry stores the token in the keychain under the widget's own service name.

---

## Milestones

### v0.1 — Static Prototype
- Scaffold Tauri v2 project (Rust backend + HTML/TS frontend).
- Set up repository structure: `.gitignore`, `LICENSE` (MIT), `package.json`, `tsconfig.json` with `strict: true`.
- Commit `Cargo.lock` and `package-lock.json` for reproducible builds.
- Enable `#![deny(warnings)]` in Rust, configure `cargo clippy` and `cargo fmt`.
- Render three pixel-art health bars with hardcoded data on an HTML Canvas.
- Validate the visual design: pixel-art hearts, bars, text at integer scale.
- Desktop window with drag support and transparent/dark background.

### v0.2 — Live Data
- Read OAuth access token from macOS Keychain (`Claude Code-credentials`).
- Fetch `GET https://api.anthropic.com/api/oauth/usage` with Bearer auth.
- Parse JSON response and map `five_hour`, `seven_day`, `seven_day_opus` to the three meters.
- Convert `resets_at` ISO timestamps to human-readable relative strings.
- Auto-refresh on configurable interval (default 5 minutes).
- Handle 401 → attempt token refresh. Show "Run `claude login`" indicator on persistent failure.
- Cache last successful response to disk for stale-data fallback.
- Dynamically show/hide meters based on which API fields are non-null.
- Never log or expose the OAuth token — mask in any debug output.

### v0.3 — Polish
- Heart icon color transitions based on usage level.
- Tooltip on hover with exact utilization value.
- Click on widget opens `https://claude.ai/settings/usage` in default browser.
- Right-click context menu: Refresh Now, Settings, Always on Top, Quit.
- Window position persistence across restarts.

### v1.0 — Release
- Settings UI: theme (dark/light), pixel scale, refresh interval, always-on-top toggle.
- System tray icon with quick-access menu.
- Auto-launch on login (optional, via macOS Launch Agent).
- Fallback setup flow for users without Claude Code (manual token paste via `claude setup-token`).
- **README.md:** screenshot/GIF, prerequisites, 3-command quick start, how it works, configuration, building from source, troubleshooting.
- **CONTRIBUTING.md:** how to contribute, code style expectations, PR process.
- **GitHub Actions CI** (`.github/workflows/build.yml`): lint (`cargo fmt --check`, `cargo clippy`), test, build on every push/PR.
- **GitHub Actions Release** workflow: on tagged `v*` push, build macOS `.dmg` and attach to GitHub Release.
- Tag `v1.0.0` and create first GitHub Release with downloadable `.dmg`.

### v1.1 — Enhancements
- System notifications at configurable usage thresholds (e.g., 80%, 95%).
- Usage trend sparkline (rolling history stored locally).
- Support for multiple Claude accounts (read multiple keychain entries).

---

## Open Questions

1. **API stability:** The `/api/oauth/usage` endpoint is undocumented — it was discovered by inspecting Claude Code's network traffic. It could change without notice. The widget should handle unexpected response shapes gracefully and log parsing errors for debugging.
2. **Token refresh endpoint:** What is the exact URL and payload format for refreshing an expired OAuth token? Claude Code handles this internally, but the widget needs to replicate it. Inspect Claude Code's network traffic or source for the refresh flow.
3. **Keychain access permissions:** On first read, macOS will prompt the user to allow the widget to access the `Claude Code-credentials` keychain entry. Is this a one-time prompt, or does it recur? If it recurs, the widget should store its own copy of the token under a separate keychain service name after the first successful read.
4. **Additional API fields:** The response includes `seven_day_oauth_apps` and `iguana_necktie` which are currently null. These may become relevant for future plan types. The widget should be designed to dynamically render any non-null meter.
5. **Rate limiting on the usage endpoint:** How frequently can this endpoint be polled? 5-minute intervals are conservative, but we should verify there's no aggressive rate limit on this specific route.
6. ~~Is there an official API endpoint?~~ **Resolved.** `GET https://api.anthropic.com/api/oauth/usage` exists and returns structured JSON.
7. ~~Authentication method?~~ **Resolved.** Piggyback on Claude Code's OAuth token stored in the macOS Keychain.

---

## Success Metrics

- **Adoption:** User installs and keeps the widget running for 7+ consecutive days.
- **Glanceability:** User can identify their usage state without clicking or reading — validated via the pixel-art color system.
- **Accuracy:** Widget data matches the Claude UI usage page within ±1% and ±1 minute of reset time.

---

## Appendix A: Recommended Build Workflow with Claude Code

### Project Setup

1. Create the project directory and initialize it:
   ```bash
   mkdir claude-usage-widget && cd claude-usage-widget
   ```

2. Place this PRD file in the project root as `PRD.md`.

3. Create a `CLAUDE.md` file in the project root. This is a special file that Claude Code reads automatically when it starts in a directory — think of it as persistent context that survives between sessions. Put the following in it:
   ```markdown
   # Claude Usage Widget

   ## Project Context
   Read PRD.md for the full product spec.

   ## Tech Stack
   - Tauri v2 (Rust backend + HTML/TS frontend)
   - macOS Keychain for auth (security-framework crate)
   - HTML Canvas for pixel art rendering (nearest-neighbor scaling, no anti-aliasing)
   - Target: macOS only for v1

   ## Key API Details
   - Endpoint: GET https://api.anthropic.com/api/oauth/usage
   - Auth: Bearer token from macOS Keychain ("Claude Code-credentials")
   - Response fields: five_hour, seven_day, seven_day_opus (each has utilization + resets_at)

   ## Current Milestone
   v0.1 — Static Prototype (hardcoded data, pixel art rendering, draggable window)
   ```

4. Open your terminal in the project directory and run `claude` to start Claude Code.

### Why CLAUDE.md, Not Pasting the PRD

Pasting the entire PRD into the chat works, but it has drawbacks: it eats your context window on every message, it can't be updated between sessions, and Claude Code can't reference it later without you re-pasting. The `CLAUDE.md` file solves all of these — Claude Code reads it on startup and uses it as ambient context. The `PRD.md` stays as a reference file that Claude Code can read when it needs detail, but the `CLAUDE.md` keeps the working instructions concise.

### IDE: It Doesn't Matter Much

Claude Code runs in your terminal, not inside an IDE. You can use it alongside VS Code, Cursor, or nothing at all — it writes files directly to disk either way. That said, having VS Code open alongside the terminal is useful for two reasons: you can see file changes in real time as Claude Code writes them, and you get syntax highlighting and linting feedback that helps you spot issues before running the app. But this is a convenience, not a requirement.

### Build Sequence

Work through the milestones in order. For each milestone, tell Claude Code what to build in plain language and let it scaffold, iterate, and test. A good workflow:

1. **v0.1:** "Initialize a Tauri v2 project. Create an HTML Canvas frontend that renders three pixel-art health bars with hardcoded data. Use the visual spec from PRD.md for the heart icons, bar colors, and text layout. The window should be draggable with a dark transparent background."

2. **v0.2:** "Add a Rust backend command that reads the OAuth token from the macOS Keychain entry named 'Claude Code-credentials', fetches GET https://api.anthropic.com/api/oauth/usage with a Bearer token, and returns the JSON to the frontend via Tauri IPC. Set up a 5-minute auto-refresh. Handle 401 errors gracefully."

3. **v0.3:** "Add hover tooltips, right-click context menu, click-to-open-browser, and save/restore window position."

4. **v1.0:** "Add a settings panel, system tray icon, and auto-launch on login."

Update the `## Current Milestone` line in `CLAUDE.md` as you complete each phase.

---

## Appendix B: Pixel Art Asset Requirements

| Asset | Size (1x) | States | Format |
|---|---|---|---|
| Heart icon | 16×16 px | Empty, 25%, 50%, 75%, Full | PNG sprite sheet or inline SVG |
| Health bar border | 200×16 px | Static | 9-slice PNG or drawn programmatically |
| Health bar fill | Variable width × 12 px | Colored per threshold | Drawn programmatically |

All pixel art must use **nearest-neighbor scaling** (no interpolation) to maintain crisp pixel edges at any display scale factor.
