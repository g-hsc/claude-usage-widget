#![deny(warnings)]
// objc 0.2's sel_impl macro internally checks cfg(cargo-clippy); allow it so
// #![deny(warnings)] doesn't turn that internal cfg probe into a compile error.
#![allow(unexpected_cfgs)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types mirroring the Anthropic usage API response
// ---------------------------------------------------------------------------

/// A single rolling usage window returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    /// Utilisation as a percentage (0–100).
    pub utilization: f64,
    /// ISO 8601 UTC timestamp when this window resets, or `null` if unknown.
    pub resets_at: Option<String>,
}

/// Full response from `GET https://api.anthropic.com/api/oauth/usage`.
/// Fields that are `null` in the response are represented as `None` here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    pub seven_day_opus: Option<UsageWindow>,
    /// Included for forward-compatibility; currently always null.
    pub seven_day_oauth_apps: Option<UsageWindow>,
}

// ---------------------------------------------------------------------------
// Keychain access
// ---------------------------------------------------------------------------

/// Read the Claude Code OAuth access token from the macOS Keychain.
///
/// Uses the system `security` CLI, which reliably accesses Keychain items in
/// both signed and unsigned (dev) builds without triggering extra permission
/// dialogs. The raw token value is never logged or included in error messages.
fn read_oauth_token() -> Result<String, String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .map_err(|_| "Could not run the `security` CLI command".to_string())?;

    if !output.status.success() {
        return Err(
            "Claude Code credentials not found. Run `claude login` in your terminal.".to_string(),
        );
    }

    let payload = std::str::from_utf8(&output.stdout)
        .map_err(|_| "Credentials contain invalid UTF-8".to_string())?
        .trim();

    let creds: serde_json::Value = serde_json::from_str(payload)
        .map_err(|_| "Malformed credentials in Keychain".to_string())?;

    creds["claudeAiOauth"]["accessToken"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Access token missing. Run `claude login` in your terminal.".to_string())
}

// ---------------------------------------------------------------------------
// API fetch
// ---------------------------------------------------------------------------

/// Fetch live usage data from the Anthropic API using the provided bearer token.
async fn fetch_usage_from_api(token: &str) -> Result<UsageResponse, String> {
    let client = reqwest::Client::new();

    let resp = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Authentication failed. Run `claude login` in your terminal.".to_string());
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("Rate limited by Anthropic API. Will retry automatically.".to_string());
    }
    if !status.is_success() {
        return Err(format!("Anthropic API returned HTTP {status}"));
    }

    resp.json::<UsageResponse>()
        .await
        .map_err(|e| format!("Failed to parse API response: {e}"))
}

// ---------------------------------------------------------------------------
// macOS window pinning
// ---------------------------------------------------------------------------

/// Pin the window to every macOS Space (including full-screen app spaces) and
/// set it to floating level so it stays above normal windows.
///
/// Flags applied via direct Objective-C call to `setCollectionBehavior:`:
///   NSWindowCollectionBehaviorCanJoinAllSpaces = 1 << 0  — appear on all Spaces
///   NSWindowCollectionBehaviorStationary       = 1 << 4  — don't animate on switch
///   NSWindowCollectionBehaviorIgnoresCycle     = 1 << 6  — skip Cmd-` cycling
///
/// Window level 3 = NSFloatingWindowLevel — floats above normal app windows.
#[cfg(target_os = "macos")]
fn apply_macos_sticky(window: &tauri::WebviewWindow, sticky: bool) {
    use objc::{msg_send, sel, sel_impl};

    let _ = window.set_always_on_top(sticky);
    let _ = window.set_visible_on_all_workspaces(sticky);

    let ns_window = match window.ns_window() {
        Ok(ptr) => ptr as *mut objc::runtime::Object,
        Err(_) => return,
    };

    const STICKY_BEHAVIOR: u64 = (1 << 0) | (1 << 4) | (1 << 6);
    const DEFAULT_BEHAVIOR: u64 = 1 << 2; // NSWindowCollectionBehaviorManaged

    let behavior: u64 = if sticky {
        STICKY_BEHAVIOR
    } else {
        DEFAULT_BEHAVIOR
    };
    // NSFloatingWindowLevel = 3; kCGNormalWindowLevel = 0
    let level: i64 = if sticky { 3 } else { 0 };

    unsafe {
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
        let _: () = msg_send![ns_window, setLevel: level];
        // Keep widget visible even when another app is in the foreground
        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_macos_sticky(window: &tauri::WebviewWindow, sticky: bool) {
    let _ = window.set_always_on_top(sticky);
}

// ---------------------------------------------------------------------------
// Tauri IPC commands
// ---------------------------------------------------------------------------

mod commands {
    use super::{apply_macos_sticky, fetch_usage_from_api, read_oauth_token, UsageResponse};

    /// Read the Keychain token and return live usage data to the frontend.
    /// Returns an actionable error string if anything goes wrong.
    #[tauri::command]
    pub async fn fetch_usage() -> Result<UsageResponse, String> {
        let token = read_oauth_token()?;
        fetch_usage_from_api(&token).await
    }

    /// Toggle sticky mode from the settings UI.
    /// Applies collection behavior + window level atomically.
    #[tauri::command]
    pub fn set_sticky(window: tauri::WebviewWindow, sticky: bool) -> Result<(), String> {
        apply_macos_sticky(&window, sticky);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

/// Initialises and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                // Apply sticky settings at launch so they take effect before the
                // first frame is drawn, regardless of the in-app toggle state.
                apply_macos_sticky(&window, true);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_usage,
            commands::set_sticky
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
