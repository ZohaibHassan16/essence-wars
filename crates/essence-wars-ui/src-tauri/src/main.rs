// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// TEMPORARILY COMMENTED OUT FOR DEBUGGING - UNCOMMENT BEFORE FINAL RELEASE
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    essence_wars_ui_lib::run()
}
