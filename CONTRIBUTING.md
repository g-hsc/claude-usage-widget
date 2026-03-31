# Contributing

Thanks for your interest in contributing to Claude Usage Widget.

## Before you start

- Open an issue before writing a large PR so we can align on the approach.
- Keep the dependency count low — every new crate or npm package needs a clear reason.
- This project targets macOS only. Don't add platform-specific workarounds for other OSes.

## Development setup

```bash
git clone https://github.com/g-hsc/claude-usage-widget.git
cd claude-usage-widget
npm install
npm run dev   # starts Tauri dev mode with hot-reload
```

You need Claude Code installed and logged in (`claude login`) for live data.

## Code standards

**Rust**
- Zero warnings: `#![deny(warnings)]` is enforced.
- Run `cargo fmt` before committing.
- Run `cargo clippy` and fix all warnings before opening a PR.
- All `#[tauri::command]` functions must have a doc comment.
- Never log, print, or expose the OAuth token.

**Frontend (HTML/TypeScript)**
- Single-file frontend (`src/index.html`) — no framework, no bundler beyond Tauri.
- No `console.log` in production code.
- All pixel-art rendering goes through the canvas with `imageSmoothingEnabled = false`.

## Pull request checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes with zero warnings
- [ ] `npm run typecheck` passes
- [ ] `npm run build` produces a working `.app`
- [ ] No hardcoded paths or personal information
- [ ] PR description explains *why*, not just *what*

## Commit messages

Use short, descriptive imperative messages:
- `Add reset position to context menu`
- `Fix window height calculation for single meter`
- `Update API beta header`

Not: `update stuff`, `fix bug`, `WIP`

## Versioning

This project uses [semver](https://semver.org). Releases are tagged `v0.x.0`.
