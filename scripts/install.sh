#!/usr/bin/env bash
# install.sh — sets up the `usage` terminal command for Claude Usage Widget.
#
# Run this once after cloning and building the project:
#   ./scripts/install.sh
#
# After running, type `usage` in any new terminal to launch the widget.

set -e

PROJ="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_APP="$PROJ/src-tauri/target/release/bundle/macos/Claude Usage Widget.app"
BIN_DIR="$HOME/bin"
USAGE_CMD="$BIN_DIR/usage"

# ── Verify a release build exists ─────────────────────────────────────────────
if [[ ! -d "$RELEASE_APP" ]]; then
  echo "No release build found. Building now (this takes ~2 minutes the first time)..."
  cd "$PROJ"
  npm run build
fi

# ── Create ~/bin if needed ─────────────────────────────────────────────────────
mkdir -p "$BIN_DIR"

# ── Write the usage command ────────────────────────────────────────────────────
cat > "$USAGE_CMD" << SCRIPT
#!/usr/bin/env bash
# Launch the Claude Usage Widget.
# Re-run scripts/install.sh after rebuilding to pick up a new release.

APP="$RELEASE_APP"

if [[ ! -d "\$APP" ]]; then
  echo "Widget not built. Run: npm run build (inside the project directory)"
  exit 1
fi

# Bring to front if already running, otherwise open fresh
if pgrep -qx "Claude Usage Widget"; then
  osascript -e 'tell application "Claude Usage Widget" to activate' 2>/dev/null || true
else
  open "\$APP"
fi
SCRIPT

chmod +x "$USAGE_CMD"

# ── Add ~/bin to PATH in shell config if not already there ────────────────────
add_to_path() {
  local rc="$1"
  if [[ -f "$rc" ]] && ! grep -q 'HOME/bin' "$rc"; then
    echo '' >> "$rc"
    echo '# User scripts' >> "$rc"
    echo 'export PATH="$HOME/bin:$PATH"' >> "$rc"
    echo "  Added ~/bin to PATH in $rc"
  fi
}

add_to_path "$HOME/.zshrc"
add_to_path "$HOME/.bashrc"
add_to_path "$HOME/.bash_profile"

echo ""
echo "✓ Installed: $USAGE_CMD"
echo ""
echo "To use it now, run:"
echo "  source ~/.zshrc   (or open a new terminal tab)"
echo "  usage"
