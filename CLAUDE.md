# Claude Usage Widget

## Project Context

This is a macOS desktop widget that displays Claude usage limits as pixel-art health bars. Read `PRD.md` for the full product spec, visual design details, and milestone breakdown.

## Tech Stack

- **Tauri v2** — Rust backend + HTML/TypeScript frontend
- **HTML Canvas** — Pixel art rendering with nearest-neighbor scaling (no anti-aliasing, integer scaling only)
- **Rust `security-framework` crate** — Read OAuth token from macOS Keychain
- **Rust `reqwest` crate** — HTTP requests to the usage API
- **Target:** macOS only

## API Details

**Endpoint:**
```
GET https://api.anthropic.com/api/oauth/usage
```

**Headers:**
```
Authorization: Bearer <token from keychain>
anthropic-beta: oauth-2025-04-20
Content-Type: application/json
Accept: application/json
```

**Response shape:**
```json
{
  "five_hour": { "utilization": 14.0, "resets_at": "2026-03-31T20:59:59Z" },
  "seven_day": { "utilization": 15.0, "resets_at": "2026-04-04T11:00:00Z" },
  "seven_day_opus": { "utilization": 8.0, "resets_at": null },
  "seven_day_oauth_apps": null,
  "iguana_necktie": null
}
```

Fields that are `null` should be hidden in the UI. Non-null fields each have `utilization` (0–100 percentage) and `resets_at` (ISO 8601 timestamp or null).

## Auth

The OAuth token is stored in the **macOS Keychain** by Claude Code under:
- **Service name:** `Claude Code-credentials`
- **Payload:** JSON with `claudeAiOauth.accessToken` (`sk-ant-oat01-...`), `refreshToken`, `expiresAt`, `subscriptionType`

The widget reads this token — no login flow, no manual setup needed if Claude Code is installed and logged in.

## Visual Design (Quick Reference)

- Three meters: Session (5hr), All Models (7-day), Opus (7-day)
- Each meter: pixel-art heart icon + health bar + "X% used" text + reset time
- Heart color shifts from red → orange → grey as usage increases
- Bar fill color: red (matching reference pixel art in PRD)
- Bar border: 2px black pixel border
- Font: pixel/monospace ("Press Start 2P", "Silkscreen", or system monospace)
- All pixel art must use nearest-neighbor scaling — no interpolation
- See PRD.md "Visual Design" and "Appendix B" for full spec and color values

## Current Milestone

**v0.2 — Live Data** ✓ complete
- Keychain read via `security-framework` crate (`Claude Code-credentials`)
- `fetch_usage` Tauri IPC command → `GET https://api.anthropic.com/api/oauth/usage`
- Frontend uses `window.__TAURI__.core.invoke` for live data
- Auto-refresh every 5 minutes; error state with actionable message on failure
- Dynamic meter visibility (only renders non-null API fields)

**Next: v0.3 — Polish**
- Hover tooltips with exact utilisation value
- Click on widget → open `https://claude.ai/settings/usage` in browser
- Right-click context menu: Refresh Now, Always on Top, Quit
- Window position persistence across restarts

## Milestone Progression

After completing each milestone, update the "Current Milestone" section above. The full milestone list is in PRD.md under "Milestones":
- v0.1 — Static Prototype (hardcoded data, pixel art, draggable window)
- v0.2 — Live Data (keychain auth, API fetch, auto-refresh, error handling)
- v0.3 — Polish (tooltips, context menu, click-to-open, position persistence)
- v1.0 — Release (settings UI, system tray, auto-launch, docs)

## Conventions

- Keep the Rust backend thin: keychain access, HTTP fetching, config persistence. All rendering logic lives in the frontend.
- Cache the last successful API response to `~/.claude-usage-widget/cache.json` so the widget can show stale data if a fetch fails.
- Config lives at `~/.claude-usage-widget/config.json`.
- Single-file frontend where possible — no framework, no build toolchain beyond what Tauri provides.

## Open Source & Distribution Guardrails

This project will be published as a public GitHub repository. Every decision should optimize for "someone clones this repo and has it running in under 5 minutes."

### Repository Structure

```
claude-usage-widget/
├── README.md              ← Setup guide, screenshots, prerequisites
├── LICENSE                 ← MIT
├── CONTRIBUTING.md         ← How to contribute, code style, PR process
├── CLAUDE.md               ← Claude Code context (this file)
├── PRD.md                  ← Full product spec
├── .gitignore              ← Rust/Tauri/Node ignores, no build artifacts
├── src-tauri/              ← Rust backend (Tauri)
│   ├── Cargo.toml
│   ├── Cargo.lock          ← Committed — reproducible builds
│   ├── src/
│   │   └── main.rs
│   └── tauri.conf.json
├── src/                    ← Frontend (HTML/TS/Canvas)
│   ├── index.html
│   ├── main.ts
│   └── style.css
├── package.json            ← Minimal — only Tauri CLI + TypeScript
├── package-lock.json       ← Committed — reproducible installs
└── .github/
    └── workflows/
        └── build.yml       ← CI: lint, build, create release artifacts
```

### README.md Requirements

The README must include, in this order:
1. **One-line description** and a screenshot/GIF of the widget running
2. **Prerequisites** — Claude Code installed and logged in (`claude login`), Rust, Node.js, Xcode Command Line Tools
3. **Quick start** — clone, install, run in 3 commands max (`npm install`, `npm run tauri dev`)
4. **How it works** — brief explanation of keychain token reuse (so users understand why no login is needed)
5. **Configuration** — where the config file lives, what's configurable
6. **Building a release binary** — `npm run tauri build` and where to find the `.dmg`
7. **Troubleshooting** — common issues (keychain permission prompt, token expired, Claude Code not logged in)
8. **License** — MIT

### Code Quality

- All Rust code must compile with zero warnings (`#![deny(warnings)]` in main.rs).
- Run `cargo clippy` before committing — no clippy warnings allowed.
- Run `cargo fmt` — consistent formatting, no style debates.
- TypeScript must compile with `strict: true` in tsconfig.json.
- No `console.log` left in production code — use a conditional debug logger.
- All Tauri IPC commands must have doc comments explaining what they do.
- Error messages shown to users must be actionable (e.g., "Run `claude login` in your terminal" not "Auth failed").

### Security

- **Never log, print, or expose the OAuth token** in console output, error messages, or UI. Mask it in debug logs (e.g., `sk-ant-oat01-****`).
- **Never commit tokens, secrets, or credentials** — .gitignore must cover all keychain exports, config files with tokens, and `.env` files.
- The widget must not make any network requests other than `GET https://api.anthropic.com/api/oauth/usage`. No telemetry, no analytics, no phoning home.
- Document the security model in the README so users know exactly what data the widget accesses.

### Dependency Policy

- **Minimal dependencies.** Every crate and npm package must justify its existence. Prefer Tauri built-ins and standard library over third-party crates.
- **Pin versions.** Commit both `Cargo.lock` and `package-lock.json` so clones get identical builds.
- **No native Node modules** that require platform-specific compilation (node-gyp). The frontend is pure HTML/TS/Canvas.

### Git Hygiene

- `.gitignore` must include: `target/`, `node_modules/`, `dist/`, `.DS_Store`, `*.dmg`, `~/.claude-usage-widget/`, and any local config/cache files.
- Commit messages should be concise and descriptive (e.g., "Add keychain read command" not "update stuff").
- Tag releases with semver: `v0.1.0`, `v0.2.0`, etc.

### CI/CD (GitHub Actions)

Set up `.github/workflows/build.yml` that:
1. Runs on push to `main` and on pull requests.
2. Installs Rust, Node.js, and Tauri CLI.
3. Runs `cargo fmt --check`, `cargo clippy`, and `cargo test`.
4. Runs `npm run build` (TypeScript compilation).
5. Runs `npm run tauri build` to produce the macOS binary.
6. On tagged releases (`v*`), creates a GitHub Release with the `.dmg` attached.

### Release Artifacts

When a version is tagged, the CI should produce:
- A `.dmg` installer for macOS (universal binary if possible: arm64 + x86_64).
- Users who don't want to build from source can download the `.dmg` from the GitHub Releases page.

