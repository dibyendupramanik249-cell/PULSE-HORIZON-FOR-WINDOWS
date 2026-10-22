// Pulse Horizon - Windows
// Android's Kotlin MainActivity becomes this. There is no Activity on Windows;
// there is a window with webviews, and the UI is HTML instead of Kotlin views.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pulse_horizon_lib::run()
}
