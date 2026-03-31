# How to Use Claude Usage Widget

A step-by-step guide from cloning the repo to seeing your usage stats.

---

## 1. Prerequisites

Install these before anything else:

**Claude Code** (required — the widget uses its stored token)
```bash
# Install Claude Code from https://claude.ai/code, then:
claude login
```

**Xcode Command Line Tools**
```bash
xcode-select --install
```

**Rust** (via rustup)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

**Node.js 18+**
```bash
# Via Homebrew:
brew install node
# Or download from https://nodejs.org
```

---

## 2. Clone & install

```bash
git clone https://github.com/g-hsc/claude-usage-widget.git
cd claude-usage-widget
npm install
```

---

## 3. Build

This compiles the Rust backend and bundles the app. Takes ~2 minutes the first time.

```bash
npm run build
```

When finished, the app is at:
```
src-tauri/target/release/bundle/macos/Claude Usage Widget.app
```

---

## 4. Launch

**Option A — Open directly:**
```bash
open "src-tauri/target/release/bundle/macos/Claude Usage Widget.app"
```

**Option B — Install the `usage` terminal command (recommended):**
```bash
./scripts/install.sh
source ~/.zshrc
```

After that, type `usage` in any terminal to launch the widget instantly.

---

## 5. Load your usage data

The widget opens showing no data. Click **↻ REFRESH** at the bottom to fetch your usage from the Anthropic API.

If you see an error:
- **"Claude Code credentials not found"** → run `claude login` in your terminal, then refresh.
- **"Rate limited"** → wait 1–2 minutes and click ↻ REFRESH again.
- **"Authentication failed"** → your token expired. Run `claude login` to renew it.

---

## 6. Using the widget

| What you see | What it means |
|---|---|
| Red/orange bar | High usage (>75%) |
| Green bar | Normal usage |
| Grey heart | Usage >90% — limit close |
| `~ RETRYING` label | Last refresh failed, showing cached data |

| Action | How |
|---|---|
| Refresh data | Click **↻ REFRESH** at the bottom |
| Move the widget | Click and drag anywhere on it |
| Open settings | Click the **⚙** gear icon (top-right) |
| Quit | Click the **×** icon (top-left), or right-click → Quit |
| Reset position | Right-click → Reset Position |

---

## 7. Settings

Click the gear icon to open settings:

- **Light mode** — switch to a light background
- **Sticky (Always on Top)** — keep the widget above all windows and visible on every macOS Space (even in full-screen apps)
- **All Models (7-day)** — show or hide the 7-day all-models meter
- **Opus (7-day)** — show or hide the 7-day Opus meter
- **Accent color** — change the heart and bar tint colour

Settings are saved automatically and persist across relaunches.

---

## 8. Staying up to date

```bash
git pull
npm run build
```

If you installed the `usage` command, re-run `./scripts/install.sh` after rebuilding so it points to the new binary.

---

## Troubleshooting

**Widget is not visible / off-screen**
Right-click the app icon in the Dock and choose **Show**, or right-click the widget and choose **Reset Position**. The widget will snap to the bottom-right corner of your screen.

**macOS shows a security warning when opening the app**
Go to **System Settings → Privacy & Security**, scroll down, and click **Open Anyway**.

**macOS asks for Keychain access**
Click **Allow**. The widget needs read access to the `Claude Code-credentials` item to authenticate with the Anthropic API.

**Build fails: "xcrun: error"**
Run `xcode-select --install` and try `npm run build` again.

**Build fails: "cargo not found"**
Run `source "$HOME/.cargo/env"` or open a new terminal and try again.
