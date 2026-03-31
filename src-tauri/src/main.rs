#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

/// Entry point — delegates to the library's run function.
fn main() {
    claude_usage_widget_lib::run()
}
